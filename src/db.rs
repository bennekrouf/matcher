use std::{collections::HashMap, sync::Arc};
use anyhow::{Context, Result as AnyhowResult};
use arrow_array::{Float32Array, RecordBatch, StringArray, FixedSizeListArray};
use arrow_array::types::Float32Type;
use arrow_schema::{DataType, Field, Schema};
use arrow::record_batch::RecordBatchIterator;
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType, Table};
use arrow_array::Array;

use crate::preprocessing::preprocess_query;
use crate::embeddings::get_embeddings;
use crate::config::Config;
use crate::config::Endpoint;
use crate::filters::extract_parameters::extract_parameters;

const VECTOR_SIZE: i32 = 384;

pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    patterns_table: Table,
    endpoints: HashMap<String, Endpoint>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
    pub text: String,
    pub description: String,
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
                    Self::initialize_table(&connection, &config.clone().unwrap().endpoints).await?;
                    connection.open_table("patterns").execute().await?
                } else {
                    return Err(e.into());
                }
            }
        };

        let endpoints = if let Some(ref config) = config {
            config.endpoints.iter()
                .map(|e| (e.id.clone(), e.clone()))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Self {
            connection,
            patterns_table,
            endpoints,
        })
    }

    async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
        println!("Generating embeddings...");

        // Define structure for pattern entries
        struct PatternEntry {
            endpoint_id: String,
            pattern: String,
            text: String,
            description: String,
            embedding: Vec<f32>,
        }

        // Prepare data for patterns table
        let mut pattern_entries = Vec::new();

        for endpoint in endpoints {
            println!("\nProcessing endpoint: {}", endpoint.id);
            for pattern in &endpoint.patterns {
                println!("  Adding pattern: '{}'", pattern);
                let embedding = get_embeddings(pattern).await?;
                pattern_entries.push(PatternEntry {
                    endpoint_id: endpoint.id.clone(),
                    pattern: pattern.clone(),
                    text: endpoint.text.clone(),
                    description: endpoint.description.clone(),
                    embedding,
                });
            }
        }

        // Create patterns table
        let patterns_schema = Arc::new(Schema::new(vec![
            Field::new("endpoint_id", DataType::Utf8, false),
            Field::new("pattern", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("description", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_SIZE,
                ),
                false,
            ),
        ]));

        let ids: Vec<&str> = pattern_entries.iter().map(|e| e.endpoint_id.as_str()).collect();
        let patterns: Vec<&str> = pattern_entries.iter().map(|e| e.pattern.as_str()).collect();
        let texts: Vec<&str> = pattern_entries.iter().map(|e| e.text.as_str()).collect();
        let descriptions: Vec<&str> = pattern_entries.iter().map(|e| e.description.as_str()).collect();

        let id_array = Arc::new(StringArray::from(ids));
        let pattern_array = Arc::new(StringArray::from(patterns));
        let text_array = Arc::new(StringArray::from(texts));
        let description_array = Arc::new(StringArray::from(descriptions));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                pattern_entries.iter().map(|e| Some(e.embedding.iter().copied().map(Some).collect::<Vec<_>>())),
                VECTOR_SIZE,
            ),
        );

        let pattern_batch = RecordBatch::try_new(
            patterns_schema.clone(),
            vec![id_array, pattern_array, text_array, description_array, vector_array],
        )?;

        match connection.create_table(
            "patterns",
            Box::new(RecordBatchIterator::new(vec![Ok(pattern_batch)], patterns_schema)),
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

    pub async fn search_similar(&self, query: &str, language: &str, limit: usize) -> AnyhowResult<Vec<SearchResult>> {
        // First preprocess the query
        let processed = preprocess_query(query, language);
        println!("\nProcessed query: '{}'", processed.cleaned_text);

        // Log extracted parameters
        for (param_name, param_value) in &processed.parameters {
            println!("Detected {}: {}", param_name, param_value);
        }

        let query_embedding = get_embeddings(&processed.cleaned_text).await?;

        // Search patterns table
        let mut results = self.patterns_table
            .vector_search(query_embedding)
            .context("Failed to create vector search")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        let mut matches = Vec::new();

        while let Some(Ok(rb)) = results.next().await {
            let endpoint_id_column = rb
                .column_by_name("endpoint_id")
                .context("endpoint_id column not found")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Failed to downcast endpoint_id column")?;

            let pattern_column = rb
                .column_by_name("pattern")
                .context("pattern column not found")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Failed to downcast pattern column")?;

            let distance_column = rb
                .column_by_name("_distance")
                .context("_distance column not found")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Failed to downcast distance column")?;

            for i in 0..pattern_column.len() {
                let pattern = pattern_column.value(i);
                let endpoint_id = endpoint_id_column.value(i);
                let similarity = 1.0 - distance_column.value(i);

                // Get endpoint data from the cached endpoints
                let endpoint = self.endpoints.get(endpoint_id)
                    .context(format!("Endpoint not found: {}", endpoint_id))?;

                // Start with parameters from preprocessing
                let mut parameters = processed.parameters.clone();

                // Add any missing parameters using pattern-based extraction
                if !parameters.is_empty() {
                    let pattern_params = extract_parameters(&processed.cleaned_text, pattern)?;
                    for (key, value) in pattern_params {
                        if !parameters.contains_key(&key) {
                            parameters.insert(key, value);
                        }
                    }
                } else {
                    parameters = extract_parameters(&processed.cleaned_text, pattern)?;
                }

                // Check if all required parameters are present
                let has_required_params = match pattern {
                    p if p.contains("{app}") => parameters.contains_key("app"),
                    p if p.contains("{email}") => parameters.contains_key("email"),
                    _ => true,
                };

                if !has_required_params {
                    continue;
                }

                matches.push(SearchResult {
                    endpoint_id: endpoint_id.to_string(),
                    pattern: pattern.to_string(),
                    similarity,
                    parameters,
                    text: endpoint.text.clone(),
                    description: endpoint.description.clone(),
                });
            }
        }

        // Sort by similarity
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        Ok(matches)
    }
}

