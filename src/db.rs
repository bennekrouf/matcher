use anyhow::{Context, Result as AnyhowResult};
use arrow::record_batch::RecordBatchIterator;
use arrow_array::types::Float32Type;
use arrow_array::Array;
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType, Table};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, sync::Arc};

use crate::config::Config;
use crate::config::Endpoint;
use crate::embeddings::get_embeddings;
use crate::extract_app_name::extract_app_name;
use crate::preprocessing::preprocess_query;

const VECTOR_SIZE: i32 = 384;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
}
pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    // endpoints_table: Table,
    patterns_table: Table,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;

        if with_init {
            if let Some(ref config) = config {
                println!("Initializing database with endpoints from config...");

                // Try to drop if exists, but don't fail if it doesn't
                match connection.drop_table("patterns").await {
                    Ok(_) => println!("Dropped existing patterns table."),
                    Err(e) => println!("Note: Couldn't drop table (might not exist): {}", e),
                }

                // connection.ensure_table_dropped("patterns").await?;
                Self::initialize_table(&connection, &config.endpoints).await?;
            }
        }

        // Try to open the table
        let patterns_table = match connection.open_table("patterns").execute().await {
            Ok(table) => table,
            Err(e) => {
                // If table doesn't exist and we have config, create it
                if e.to_string().contains("Table not found") && config.is_some() {
                    println!("Table not found, creating new one...");
                    Self::initialize_table(&connection, &config.unwrap().endpoints).await?;
                    connection.open_table("patterns").execute().await?
                } else {
                    return Err(e.into());
                }
            }
        };

        Ok(Self {
            connection,
            patterns_table,
        })
    }

    async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
        println!("Generating embeddings...");

        // Prepare data for patterns table
        let mut pattern_entries = Vec::new();
        let mut pattern_embeddings = Vec::new();

        for endpoint in endpoints {
            println!("\nProcessing endpoint: {}", endpoint.id);
            for pattern in &endpoint.patterns {
                println!("  Adding pattern: '{}'", pattern);
                let embedding = get_embeddings(pattern).await?;
                pattern_entries.push((endpoint.id.clone(), pattern.clone()));
                pattern_embeddings.push(embedding);
            }
        }

        // Create patterns table
        let patterns_schema = Arc::new(Schema::new(vec![
            Field::new("endpoint_id", DataType::Utf8, false),
            Field::new("pattern", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_SIZE,
                ),
                false,
            ),
        ]));

        let ids: Vec<&str> = pattern_entries.iter().map(|(id, _)| id.as_str()).collect();
        let patterns: Vec<&str> = pattern_entries.iter().map(|(_, p)| p.as_str()).collect();

        let id_array = Arc::new(StringArray::from(ids));
        let pattern_array = Arc::new(StringArray::from(patterns));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                pattern_embeddings
                    .iter()
                    .map(|vec| Some(vec.iter().copied().map(Some).collect::<Vec<_>>())),
                VECTOR_SIZE,
            ),
        );

        let pattern_batch = RecordBatch::try_new(
            patterns_schema.clone(),
            vec![id_array, pattern_array, vector_array],
        )?;

        match connection
            .create_table(
                "patterns",
                Box::new(RecordBatchIterator::new(
                    vec![Ok(pattern_batch)],
                    patterns_schema,
                )),
            )
            .execute()
            .await
        {
            Ok(_) => println!("Table created successfully!"),
            Err(e) => {
                println!("Error creating table: {}", e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    pub async fn search_similar(
        &self,
        query: &str,
        language: &str,
        limit: usize,
    ) -> AnyhowResult<Vec<SearchResult>> {
        let processed = preprocess_query(query, language);
        println!("\nProcessed query: '{}'", processed.cleaned_text);

        for (param_name, param_value) in &processed.parameters {
            println!("!!! Detected {}: {}", param_name, param_value);
        }

        let query_embedding = get_embeddings(&processed.cleaned_text).await?;
        println!("Generated query embedding, starting vector search...");

        let mut results = self
            .patterns_table
            .vector_search(query_embedding)
            .context("Failed to create vector search")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        println!("Vector search completed, processing results...");
        let mut matches = Vec::new();

        while let Some(Ok(rb)) = results.next().await {
            println!("Processing record batch...");

            let endpoint_id_column = rb
                .column_by_name("endpoint_id")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let pattern_column = rb
                .column_by_name("pattern")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let distance_column = rb
                .column_by_name("_distance")
                .unwrap()
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap();

            println!("Found {} results in batch", pattern_column.len());

            for i in 0..pattern_column.len() {
                let pattern = pattern_column.value(i);
                let endpoint_id = endpoint_id_column.value(i);
                let similarity = 1.0 - distance_column.value(i);

                println!(
                    "Processing result {}: pattern='{}', endpoint='{}', similarity={}",
                    i, pattern, endpoint_id, similarity
                );

                let mut parameters = processed.parameters.clone();

                if !parameters.is_empty() {
                    let pattern_params = extract_parameters(&processed.cleaned_text, pattern)?;
                    for (key, value) in pattern_params {
                        if !parameters.contains_key(&key) {
                            println!("Adding pattern param: {}={}", key, value);
                            parameters.insert(key, value);
                        }
                    }
                } else {
                    parameters = extract_parameters(&processed.cleaned_text, pattern)?;
                    println!("Extracted parameters from pattern: {:?}", parameters);
                }

                let has_required_params = match pattern {
                    p if p.contains("{app}") => parameters.contains_key("app"),
                    p if p.contains("{email}") => parameters.contains_key("email"),
                    _ => true,
                };

                println!("Has required params: {}", has_required_params);

                if !has_required_params {
                    println!("Skipping result due to missing required parameters");
                    continue;
                }

                println!("Adding match to results");
                matches.push(SearchResult {
                    endpoint_id: endpoint_id.to_string(),
                    pattern: pattern.to_string(),
                    similarity,
                    parameters,
                });
            }
        }

        println!("Final matches count: {}", matches.len());
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        Ok(matches)
    }
}

fn extract_parameters(query: &str, pattern: &str) -> AnyhowResult<HashMap<String, String>> {
    let mut params = HashMap::new();

    // Check for email parameter
    if pattern.contains("{email}") {
        if let Some(email) = EMAIL_REGEX.find(query) {
            params.insert("email".to_string(), email.as_str().to_string());
        }
    }

    // Check for app parameter
    if pattern.contains("{app}") {
        if let Some(app) = extract_app_name(query) {
            params.insert("app".to_string(), app);
        }
    }

    Ok(params)
}
