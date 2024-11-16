use anyhow::Result as AnyhowResult;

use crate::database::SearchResult;
use crate::filters::extract_parameters::extract_parameters;
use crate::preprocessing::ProcessedQuery;

pub struct SearchAttempt {
    pub result: Option<SearchResult>,
    pub similarity: f32, // Always include the similarity score
}

pub async fn process_single_result(
    index: usize,
    pattern: &str,
    endpoint_id: &str,
    distance: f32,
    processed: &ProcessedQuery,
) -> AnyhowResult<SearchAttempt> {
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

    let result = if has_required_params {
        println!("Adding match to results");
        Some(SearchResult {
            endpoint_id: endpoint_id.to_string(),
            pattern: pattern.to_string(),
            similarity,
            parameters,
        })
    } else {
        println!("Skipping result due to missing required parameters");
        None
    };

    Ok(SearchAttempt { result, similarity })
}
