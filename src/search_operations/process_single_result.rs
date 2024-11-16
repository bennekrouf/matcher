use crate::config::Config;
use crate::database::SearchResult;
use crate::filters::extract_parameters::extract_parameters;
use crate::preprocessing::ProcessedQuery;
use anyhow::Result as AnyhowResult;
use tracing::{debug, info};

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
    config: &Config,
) -> AnyhowResult<SearchAttempt> {
    let similarity = 1.0 - distance;
    info!(
        "Processing result {}: pattern='{}', endpoint='{}', similarity={}",
        index, pattern, endpoint_id, similarity
    );

    let endpoint = config
        .endpoints
        .iter()
        .find(|e| e.id == endpoint_id)
        .ok_or_else(|| anyhow::anyhow!("Endpoint not found: {}", endpoint_id))?;

    let mut parameters = processed.parameters.clone();
    debug!("Provided parameters: {:?}", parameters);

    if !parameters.is_empty() {
        let pattern_params = extract_parameters(&processed.cleaned_text, pattern)?;
        debug!("Extracted parameters from pattern: {:?}", pattern_params);
        for (key, value) in pattern_params {
            if !parameters.contains_key(&key) {
                debug!("Adding pattern param: {}={}", key, value);
                parameters.insert(key, value);
            }
        }
    } else {
        parameters = extract_parameters(&processed.cleaned_text, pattern)?;
        debug!("Extracted parameters from pattern: {:?}", parameters);
    }

    let parameter_analysis = endpoint.analyze_parameters(&parameters);
    debug!("Parameter analysis: {:?}", parameter_analysis);

    let result = Some(SearchResult {
        endpoint_id: endpoint_id.to_string(),
        pattern: pattern.to_string(),
        similarity,
        parameters,
        parameter_analysis: Some(parameter_analysis),
    });

    Ok(SearchAttempt { result, similarity })
}
