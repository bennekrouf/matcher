use std::sync::Arc;
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

const VECTOR_SIZE: i32 = 384;

pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    table: Table,
}

#[derive(Debug)]
pub struct SearchResult {
    #[allow(dead_code)]
    pub id: String,
    pub text: String,
    pub similarity: f32,
    #[allow(dead_code)]
    pub distance: f32,
    #[allow(dead_code)]
    pub rank: usize,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;

        if with_init {
            if let Some(config) = config {
                println!("Initializing database with endpoints from config...");
                Self::initialize_table(&connection, &config.endpoints).await?;
            }
        }

        let table = connection.open_table("endpoints").execute().await?;

        Ok(Self {
            connection,
            table,
        })
    }

    async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
        println!("Generating embeddings...");

        // Collect all texts and their corresponding IDs
        let mut all_entries: Vec<(String, String)> = Vec::new();

        for endpoint in endpoints {
            // Use all_texts() to get all variations
            for text in endpoint.all_texts() {
                all_entries.push((endpoint.id.clone(), text));
            }
        }

        // Generate embeddings for all texts
        let mut embeddings = Vec::new();
        for (_, text) in &all_entries {
            let embedding = get_embeddings(text).await?;
            embeddings.push(embedding);
        }

        // Rest of the code remains the same...
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_SIZE,
                ),
                false,
            ),
        ]));

        let ids: Vec<&str> = all_entries.iter().map(|(id, _)| id.as_str()).collect();
        let texts: Vec<&str> = all_entries.iter().map(|(_, text)| text.as_str()).collect();

        let id_array = Arc::new(StringArray::from(ids));
        let text_array = Arc::new(StringArray::from(texts));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                embeddings.iter().map(|vec| Some(vec.iter().copied().map(Some).collect::<Vec<_>>())),
                VECTOR_SIZE,
            ),
        );

        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![id_array, text_array, vector_array],
        )?;

        connection.create_table(
            "endpoints",
            Box::new(RecordBatchIterator::new(vec![Ok(record_batch)], schema)),
        )
        .execute()
        .await?;

        println!("Table created successfully with {} entries!", all_entries.len());
        Ok(())
    }

    pub async fn search_similar(&self, query: &str, language: &str, limit: usize) -> AnyhowResult<Vec<SearchResult>> {
        // Preprocess the query
        let processed_query = preprocess_query(query, language);
        let query_embedding = get_embeddings(&processed_query).await?;
        let mut results = Vec::new();

        let mut search_results = self.table
            .vector_search(query_embedding)
            .context("Failed to create vector search")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        while let Some(Ok(rb)) = search_results.next().await {
            let text_column = rb
                .column_by_name("text")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let id_column = rb
                .column_by_name("id")
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

            for i in 0..text_column.len() {
                let text = text_column.value(i).to_string();
                let id = id_column.value(i).to_string();
                let distance = distance_column.value(i);
                let similarity = 1.0 - distance;

                results.push(SearchResult {
                    id,
                    text,
                    similarity,
                    distance,
                    rank: i + 1,
                });
            }
        }
        Ok(results)
    }
}
