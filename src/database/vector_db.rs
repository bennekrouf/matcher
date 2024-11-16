// src/database/vector_db.rs
use anyhow::{Context, Result as AnyhowResult};
use arrow_array::{Array, Float32Array, StringArray};
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType, Table};

use crate::config::Config;
use crate::embeddings::get_embeddings;
use crate::preprocessing::preprocess_query;

use super::db_initializer::initialize_table;
use super::parameter_extractor::extract_parameters;
use super::search_result::SearchResult;

pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    patterns_table: Table,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;

        if with_init {
            if let Some(ref config) = config {
                println!("Initializing database with endpoints from config...");

                match connection.drop_table("patterns").await {
                    Ok(_) => println!("Dropped existing patterns table."),
                    Err(e) => println!("Note: Couldn't drop table (might not exist): {}", e),
                }

                initialize_table(&connection, &config.endpoints).await?;
            }
        }

        let patterns_table = match connection.open_table("patterns").execute().await {
            Ok(table) => table,
            Err(e) => {
                if e.to_string().contains("Table not found") && config.is_some() {
                    println!("Table not found, creating new one...");
                    initialize_table(&connection, &config.unwrap().endpoints).await?;
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
