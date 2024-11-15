use super::db::VectorDB;
use super::search_result::{MatchResult, SearchResult};
use super::types::{extract_columns, BatchColumns};
use crate::embeddings::get_embeddings;
use crate::filters::extract_parameters;
use crate::preprocessing::{preprocess_query, ProcessedQuery};
use crate::vector_db::search::extract_parameters::extract_parameters;
use anyhow::{Context, Result as AnyhowResult};
use arrow_array::Array;
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::DistanceType;

impl VectorDB {
    pub async fn search_similar(
        &self,
        query: &str,
        language: &str,
        limit: usize,
    ) -> AnyhowResult<Vec<MatchResult>> {
        let processed = preprocess_query(query, language);
        let query_embedding = get_embeddings(&processed.cleaned_text).await?;
        let results = self.perform_vector_search(query_embedding, limit).await?;
        let mut matches = self.process_search_results(results, &processed).await?;
        matches.sort_by(|a, b| b.similarity().partial_cmp(&a.similarity()).unwrap());
        Ok(matches)
    }

    async fn perform_vector_search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> AnyhowResult<arrow::record_batch::RecordBatch> {
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

    async fn process_search_results(
        &self,
        record_batch: arrow::record_batch::RecordBatch,
        processed_query: &ProcessedQuery,
    ) -> AnyhowResult<Vec<MatchResult>> {
        let columns = extract_columns(&record_batch)?;
        let mut matches = Vec::new();

        for i in 0..columns.pattern_column.len() {
            if let Some(match_result) = self
                .process_single_result(&columns, i, processed_query)
                .await?
            {
                matches.push(match_result);
            }
        }

        Ok(matches)
    }

    async fn process_single_result(
        &self,
        columns: &BatchColumns<'_>,
        index: usize,
        processed_query: &ProcessedQuery,
    ) -> AnyhowResult<Option<MatchResult>> {
        let pattern = columns.pattern_column.value(index);
        let endpoint_id = columns.endpoint_id_column.value(index);
        let similarity = 1.0 - columns.distance_column.value(index);

        // First check similarity threshold before anything else
        if similarity < 0.7 {
            return Ok(None);
        }

        let endpoint = self
            .endpoints
            .get(endpoint_id)
            .context(format!("Endpoint not found: {}", endpoint_id))?;

        // Extract parameters
        let mut parameters = processed_query.parameters.clone();
        if !parameters.is_empty() {
            let pattern_params = extract_parameters(&processed_query.cleaned_text, pattern)?;
            parameters.extend(pattern_params);
        } else {
            parameters = extract_parameters(&processed_query.cleaned_text, pattern)?;
        }

        let result = SearchResult {
            endpoint_id: endpoint_id.to_string(),
            pattern: pattern.to_string(),
            similarity,
            parameters: parameters.clone(),
            text: endpoint.text.clone(),
            description: endpoint.description.clone(),
        };

        // Check for required parameters only AFTER we've found a good semantic match
        let mut missing_params = Vec::new();
        for param in &endpoint.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                missing_params.push(param.name.clone());
            }
        }

        // Always return the match, just mark it as partial if parameters are missing
        Ok(Some(if missing_params.is_empty() {
            MatchResult::Complete(result)
        } else {
            MatchResult::Partial {
                result,
                missing_params,
            }
        }))
    }
}
