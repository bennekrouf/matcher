use anyhow::Result as AnyhowResult;
use arrow_array::{Array, Float32Array, RecordBatch, StringArray};

use super::process_single_result::process_single_result;
use crate::database::SearchResult;
use crate::preprocessing::ProcessedQuery;

pub struct SearchAttempt {
    pub result: Option<SearchResult>,
    pub similarity: f32, // Always include the similarity score
}

pub async fn process_search_batch(
    rb: RecordBatch,
    processed: &ProcessedQuery,
) -> AnyhowResult<(Vec<SearchResult>, f32)> {
    println!("Processing record batch...");

    //let mut matches: Vec<SearchResult> = Vec::new();
    let mut best_similarity: f32 = 0.0;

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
    let mut matches = Vec::new();

    for i in 0..pattern_column.len() {
        let similarity = 1.0 - distance_column.value(i);
        best_similarity = best_similarity.max(similarity);

        let result = process_single_result(
            i,
            pattern_column.value(i),
            endpoint_id_column.value(i),
            similarity,
            processed,
        )
        .await?;

        matches.extend(result.result);
    }

    Ok((matches, best_similarity))
}
