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

use crate::embeddings::get_embeddings;

const VECTOR_SIZE: i32 = 384;

pub struct VectorDB {
    connection: Connection,
    table: Table,
}

impl VectorDB {
    pub async fn new(db_path: &str, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;
        
        if with_init {
            println!("Initializing database with sample data...");
            Self::initialize_table(&connection).await?;
        }
        
        let table = connection.open_table("endpoints").execute().await?;
        
        Ok(Self {
            connection,
            table,
        })
    }

    async fn initialize_table(connection: &Connection) -> AnyhowResult<()> {
        let texts = vec![
            ("endpoint1", "run analysis"),
            ("endpoint2", "perform calculation"),
        ];

        println!("Generating embeddings...");
        let mut embeddings = Vec::new();
        for (_, text) in &texts {
            let embedding = get_embeddings(text).await?;
            embeddings.push(embedding);
        }

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

        let ids: Vec<&str> = texts.iter().map(|(id, _)| *id).collect();
        let texts_data: Vec<&str> = texts.iter().map(|(_, text)| *text).collect();

        let id_array = Arc::new(StringArray::from(ids));
        let text_array = Arc::new(StringArray::from(texts_data));
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

        println!("Table created successfully!");
        Ok(())
    }

    pub async fn search_similar(&self, query: &str, limit: usize) -> AnyhowResult<()> {
        let query_embedding = get_embeddings(query).await?;

        let mut results = self.table
            .vector_search(query_embedding)
            .context("Failed to create vector search")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        while let Some(Ok(rb)) = results.next().await {
            let text_column = rb
                .column_by_name("text")
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
                let matched_text = text_column.value(i);
                let distance = distance_column.value(i);
                let similarity = 1.0 - distance;

                println!("Matched text: {} (similarity: {:.2})", matched_text, similarity);
            }
        }
        Ok(())
    }
}
