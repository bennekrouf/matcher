use super::db::VectorDB;
use crate::config::{Config, Endpoint};
use crate::embeddings::get_embeddings;
use anyhow::{Context, Result as AnyhowResult};
use arrow::record_batch::RecordBatchIterator;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::Connection;
use lancedb::Table;
use std::sync::Arc;

const VECTOR_SIZE: i32 = 384;

struct PatternEntry {
    endpoint_id: String,
    pattern: String,
    text: String,
    description: String,
    embedding: Vec<f32>,
}

impl VectorDB {
    pub(crate) async fn handle_initialization(
        connection: &Connection,
        config: &Config,
    ) -> AnyhowResult<()> {
        println!("Initializing database with endpoints from config...");
        match connection.drop_table("patterns").await {
            Ok(_) => println!("Dropped existing patterns table."),
            Err(e) => println!("Note: Couldn't drop table (might not exist): {}", e),
        }
        Self::initialize_table(connection, &config.endpoints).await
    }

    pub async fn get_or_create_table(
        connection: &Connection,
        config: Option<&Config>,
    ) -> AnyhowResult<Table> {
        match connection.open_table("patterns").execute().await {
            Ok(table) => Ok(table),
            Err(e) if e.to_string().contains("Table not found") && config.is_some() => {
                println!("Table not found, creating new one...");
                Self::initialize_table(connection, &config.unwrap().endpoints).await?;
                Ok(connection.open_table("patterns").execute().await?)
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn create_pattern_entries(endpoints: &[Endpoint]) -> AnyhowResult<Vec<PatternEntry>> {
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

        Ok(pattern_entries)
    }

    fn create_patterns_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
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
        ]))
    }

    fn create_record_batch(
        pattern_entries: &[PatternEntry],
        schema: &Arc<Schema>,
    ) -> AnyhowResult<RecordBatch> {
        let ids: Vec<&str> = pattern_entries
            .iter()
            .map(|e| e.endpoint_id.as_str())
            .collect();
        let patterns: Vec<&str> = pattern_entries.iter().map(|e| e.pattern.as_str()).collect();
        let texts: Vec<&str> = pattern_entries.iter().map(|e| e.text.as_str()).collect();
        let descriptions: Vec<&str> = pattern_entries
            .iter()
            .map(|e| e.description.as_str())
            .collect();

        let id_array = Arc::new(StringArray::from(ids));
        let pattern_array = Arc::new(StringArray::from(patterns));
        let text_array = Arc::new(StringArray::from(texts));
        let description_array = Arc::new(StringArray::from(descriptions));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                pattern_entries
                    .iter()
                    .map(|e| Some(e.embedding.iter().copied().map(Some).collect::<Vec<_>>())),
                VECTOR_SIZE,
            ),
        );

        RecordBatch::try_new(
            schema.clone(),
            vec![
                id_array,
                pattern_array,
                text_array,
                description_array,
                vector_array,
            ],
        )
        .context("Failed to create record batch")
    }

    async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
        println!("Generating embeddings...");
        let pattern_entries = Self::create_pattern_entries(endpoints).await?;
        let schema = Self::create_patterns_schema();
        let batch = Self::create_record_batch(&pattern_entries, &schema)?;
        Self::create_table(connection, schema, batch).await
    }

    async fn create_table(
        connection: &Connection,
        schema: Arc<Schema>,
        batch: RecordBatch,
    ) -> AnyhowResult<()> {
        match connection
            .create_table(
                "patterns",
                Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema)),
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
}
