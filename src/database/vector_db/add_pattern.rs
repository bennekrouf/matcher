use super::super::initialization::VECTOR_SIZE;
use super::db::VectorDB;
use crate::candle::get_embeddings::get_embeddings;
use crate::config::Endpoint;
use anyhow::Result as AnyhowResult;
use arrow::datatypes::Float32Type;
use arrow_array::{FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use std::sync::Arc;

impl VectorDB {
    pub async fn add_pattern(&self, endpoint_id: &str, pattern: &str) -> AnyhowResult<()> {
        let embedding = get_embeddings(pattern).await?;
        let id_array = Arc::new(StringArray::from(vec![endpoint_id]));
        let pattern_array = Arc::new(StringArray::from(vec![pattern]));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                vec![Some(
                    embedding.iter().copied().map(Some).collect::<Vec<_>>(),
                )],
                VECTOR_SIZE,
            ),
        );
        let pattern_batch = RecordBatch::try_new(
            self.patterns_schema.clone(),
            vec![id_array, pattern_array, vector_array],
        )?;

        let batch_iterator =
            RecordBatchIterator::new(vec![Ok(pattern_batch)], self.patterns_schema.clone());
        self.patterns_table
            .add(Box::new(batch_iterator))
            .execute()
            .await?;

        Ok(())
    }

    pub(crate) async fn add_patterns(&self, endpoints: &[Endpoint]) -> AnyhowResult<()> {
        for endpoint in endpoints {
            println!("\nProcessing endpoint: {}", endpoint.id);
            for pattern in &endpoint.patterns {
                println!("  Adding pattern: '{}'", pattern);
                self.add_pattern(&endpoint.id, pattern).await?;
            }
        }
        Ok(())
    }
}
