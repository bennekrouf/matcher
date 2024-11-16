use crate::config::Endpoint;
use crate::embeddings::get_embeddings;
use anyhow::Result as AnyhowResult;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::Connection;
use std::sync::Arc;

use arrow::record_batch::RecordBatchIterator;
const VECTOR_SIZE: i32 = 384;

pub async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
    println!("Generating embeddings...");

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
