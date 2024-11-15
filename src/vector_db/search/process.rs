use crate::preprocessing::ProcessedQuery;
use crate::vector_db::db::VectorDB;
use crate::vector_db::search_result::MatchResult;
use crate::vector_db::types::{extract_columns, BatchColumns};
use anyhow::Result as AnyhowResult;
use arrow::record_batch::RecordBatch;
use arrow_array::Array;
use tracing::{debug, info};

impl VectorDB {
    #[tracing::instrument(skip(self, record_batch, processed_query))]
    pub(crate) async fn process_search_results(
        &self,
        record_batch: RecordBatch,
        processed_query: &ProcessedQuery,
    ) -> AnyhowResult<Vec<MatchResult>> {
        let columns = extract_columns(&record_batch)?;
        let mut matches = Vec::new();

        debug!(
            total_patterns = columns.pattern_column.len(),
            query = %processed_query.cleaned_text,
            "Starting to process patterns"
        );

        for i in 0..columns.pattern_column.len() {
            let pattern = columns.pattern_column.value(i);
            let similarity = 1.0 - columns.distance_column.value(i);

            debug!(
                index = i,
                pattern = %pattern,
                similarity = %similarity,
                "Processing pattern"
            );

            if let Some(match_result) = self
                .process_single_result(&columns, i, processed_query)
                .await?
            {
                debug!(
                    pattern = %pattern,
                    similarity = %similarity,
                    "Found valid match"
                );
                matches.push(match_result);
            } else {
                debug!(
                    pattern = %pattern,
                    similarity = %similarity,
                    "Pattern rejected"
                );
            }
        }

        debug!(
            matches_found = matches.len(),
            query = %processed_query.cleaned_text,
            "Completed processing"
        );

        Ok(matches)
    }
}
