use anyhow::Result as AnyhowResult;
use arrow_array::{Array, Float32Array, RecordBatch, StringArray};

use crate::database::{parameter_extractor::extract_parameters, SearchResult};
use crate::preprocessing::ProcessedQuery;

pub async fn process_search_batch(
    rb: RecordBatch,
    processed: &ProcessedQuery,
) -> AnyhowResult<Vec<SearchResult>> {
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
    let mut matches = Vec::new();

    for i in 0..pattern_column.len() {
        if let Some(result) = process_single_result(
            i,
            pattern_column.value(i),
            endpoint_id_column.value(i),
            distance_column.value(i),
            processed,
        )
        .await?
        {
            matches.push(result);
        }
    }

    Ok(matches)
}

async fn process_single_result(
    index: usize,
    pattern: &str,
    endpoint_id: &str,
    distance: f32,
    processed: &ProcessedQuery,
) -> AnyhowResult<Option<SearchResult>> {
    let similarity = 1.0 - distance;

    println!(
        "Processing result {}: pattern='{}', endpoint='{}', similarity={}",
        index, pattern, endpoint_id, similarity
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
        return Ok(None);
    }

    println!("Adding match to results");
    Ok(Some(SearchResult {
        endpoint_id: endpoint_id.to_string(),
        pattern: pattern.to_string(),
        similarity,
        parameters,
    }))
}
