use anyhow::Result as AnyhowResult;
use arrow_array::{Array, RecordBatch};

use crate::{
    config::{Config, ProcessedQuery, SearchResult},
    filters::extract_parameters::extract_parameters,
};

pub async fn process_search_batch(
    batch: RecordBatch,
    processed: &ProcessedQuery,
    config: &Config,
) -> AnyhowResult<(Vec<SearchResult>, f32)> {
    let mut results = Vec::new();
    let mut best_similarity: f32 = 0.0;

    // Get column arrays from the batch with correct column names
    let pattern_array = batch
        .column_by_name("pattern")
        .ok_or_else(|| anyhow::anyhow!("Missing pattern column"))?;
    let endpoint_id_array = batch
        .column_by_name("endpoint_id")
        .ok_or_else(|| anyhow::anyhow!("Missing endpoint_id column"))?;
    let distance_array = batch
        .column_by_name("_distance") // Changed from "distance" to "_distance"
        .ok_or_else(|| anyhow::anyhow!("Missing _distance column"))?;

    for row_idx in 0..batch.num_rows() {
        let pattern = pattern_array
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Failed to get pattern as string"))?
            .value(row_idx);

        let endpoint_id = endpoint_id_array
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Failed to get endpoint_id as string"))?
            .value(row_idx);

        let distance = distance_array
            .as_any()
            .downcast_ref::<arrow::array::Float32Array>()
            .ok_or_else(|| anyhow::anyhow!("Failed to get _distance as float"))?
            .value(row_idx);

        let endpoint = config
            .endpoints
            .iter()
            .find(|e| e.id == endpoint_id)
            .ok_or_else(|| anyhow::anyhow!("Endpoint not found: {}", endpoint_id))?;

        let mut parameters = processed.parameters.clone();
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

        let parameter_analysis = endpoint.analyze_parameters(&parameters);

        let similarity = 1.0 - distance;
        best_similarity = best_similarity.max(similarity);

        results.push(SearchResult {
            endpoint_id: endpoint_id.to_string(),
            pattern: pattern.to_string(),
            similarity,
            parameters,
            parameter_analysis,
        });
    }

    Ok((results, best_similarity))
}
