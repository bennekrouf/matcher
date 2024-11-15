use crate::embeddings::get_embeddings;
use crate::preprocessing::preprocess_query;
use crate::vector_db::db::VectorDB;
use crate::vector_db::search_result::MatchResult;
use anyhow::Result as AnyhowResult;
use tracing::{debug, warn};

impl VectorDB {
    #[tracing::instrument(skip(self))]
    pub async fn search_similar(
        &self,
        query: &str,
        language: &str,
        limit: usize,
    ) -> AnyhowResult<Vec<MatchResult>> {
        let processed = preprocess_query(query, language);
        debug!(
            original_query = %query,
            cleaned_text = %processed.cleaned_text,
            initial_parameters = ?processed.parameters,
            "Query preprocessing"
        );

        let query_embedding = get_embeddings(&processed.cleaned_text).await?;
        debug!(
            embedding_size = query_embedding.len(),
            "Generated embeddings"
        );

        debug!(%limit, "Starting vector search");

        let results = self.perform_vector_search(query_embedding, limit).await?;
        debug!(raw_results = results.num_rows(), "Vector search completed");

        let mut matches = self.process_search_results(results, &processed).await?;
        matches.sort_by(|a, b| b.similarity().partial_cmp(&a.similarity()).unwrap());

        if matches.is_empty() {
            warn!(
                query = %query,
                cleaned = %processed.cleaned_text,
                "No matches found after processing"
            );
        }
        Ok(matches)
    }
}
