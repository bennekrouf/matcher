use crate::vector_db::db::VectorDB;
use anyhow::{Context, Result as AnyhowResult};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::DistanceType;
use tracing::debug;

impl VectorDB {
    #[tracing::instrument(skip(self, query_embedding))]
    pub(crate) async fn perform_vector_search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> AnyhowResult<RecordBatch> {
        debug!(%limit, "Starting vector search");

        let mut results = self
            .patterns_table
            .vector_search(query_embedding)?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        results
            .next()
            .await
            .transpose()?
            .context("No results found")
    }
}
