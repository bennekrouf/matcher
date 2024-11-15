use crate::config::Endpoint;
use crate::filters::extract_parameters;
use crate::preprocessing::ProcessedQuery;
use crate::vector_db::db::VectorDB;
use crate::vector_db::search::result_processor::extract_parameters::extract_parameters;
use crate::vector_db::search_result::{MatchResult, SearchResult};
use crate::vector_db::types::BatchColumns;
use anyhow::{Context, Result as AnyhowResult};
use tracing::debug;
impl VectorDB {
    #[tracing::instrument(skip(self, columns, processed_query))]
    pub(crate) async fn process_single_result(
        &self,
        columns: &BatchColumns<'_>,
        index: usize,
        processed_query: &ProcessedQuery,
    ) -> AnyhowResult<Option<MatchResult>> {
        let pattern = columns.pattern_column.value(index);
        let endpoint_id = columns.endpoint_id_column.value(index);
        let similarity = 1.0 - columns.distance_column.value(index);

        debug!(
            %pattern,
            %endpoint_id,
            %similarity,
            threshold = 0.7,
            "Evaluating match"
        );

        if similarity < 0.5 {
            debug!("Similarity below threshold");
            return Ok(None);
        }

        let endpoint = self
            .endpoints
            .get(endpoint_id)
            .context(format!("Endpoint not found: {}", endpoint_id))?;

        let result =
            self.create_search_result(endpoint_id, pattern, similarity, processed_query, endpoint)?;

        let missing_params = self.check_missing_parameters(&result, endpoint);

        debug!(
            pattern = %pattern,
            has_missing = !missing_params.is_empty(),
            missing_count = missing_params.len(),
            "Processed result"
        );

        Ok(Some(if missing_params.is_empty() {
            MatchResult::Complete(result)
        } else {
            MatchResult::Partial {
                result,
                missing_params,
            }
        }))
    }

    fn create_search_result(
        &self,
        endpoint_id: &str,
        pattern: &str,
        similarity: f32,
        processed_query: &ProcessedQuery,
        endpoint: &Endpoint,
    ) -> AnyhowResult<SearchResult> {
        let mut parameters = processed_query.parameters.clone();
        if !parameters.is_empty() {
            let pattern_params = extract_parameters(&processed_query.cleaned_text, pattern)?;
            parameters.extend(pattern_params);
        } else {
            parameters = extract_parameters(&processed_query.cleaned_text, pattern)?;
        }

        debug!(?parameters, "Extracted parameters");

        Ok(SearchResult {
            endpoint_id: endpoint_id.to_string(),
            pattern: pattern.to_string(),
            similarity,
            parameters,
            text: endpoint.text.clone(),
            description: endpoint.description.clone(),
        })
    }

    fn check_missing_parameters(&self, result: &SearchResult, endpoint: &Endpoint) -> Vec<String> {
        let mut missing_params = Vec::new();
        for param in &endpoint.parameters {
            if param.required && !result.parameters.contains_key(&param.name) {
                missing_params.push(param.name.clone());
            }
        }
        debug!(?missing_params, "Checked required parameters");
        missing_params
    }
}
