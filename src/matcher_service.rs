use crate::config::Config;
use crate::process_search_results;
use crate::vector_db::db::VectorDB;
use crate::vector_db::search_result::MatchResult;
use anyhow::Result as AnyhowResult;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct MatcherEngine {
    config: Arc<Config>,
    db: Arc<VectorDB>,
}

impl MatcherEngine {
    pub async fn new(config: Arc<Config>) -> AnyhowResult<Self> {
        info!("Initializing VectorDB");
        let db = VectorDB::new("data/mydb", Some((*config).clone()), false)
            .await
            .map(Arc::new)?;

        Ok(Self { config, db })
    }

    pub async fn find_matches(
        &self,
        query: &str,
        language: &str,
        limit: usize,
    ) -> AnyhowResult<Vec<MatchResult>> {
        let results = self.db.search_similar(query, language, limit).await?;

        if results.is_empty() {
            warn!("No matches found for query: {}", query);
        } else {
            debug!("Found {} matches", results.len());
            for (i, result) in results.iter().enumerate() {
                match result {
                    MatchResult::Complete(r) => {
                        debug!(
                            "Match {} (Complete): id={}, similarity={:.4}",
                            i + 1,
                            r.endpoint_id,
                            r.similarity
                        );
                    }
                    MatchResult::Partial {
                        result: r,
                        missing_params,
                    } => {
                        debug!(
                            "Match {} (Partial): id={}, similarity={:.4}, missing: {:?}",
                            i + 1,
                            r.endpoint_id,
                            r.similarity,
                            missing_params
                        );
                    }
                }
            }
        }

        // Process best match if available
        if let Some(best_match) = results.first() {
            info!("Processing best match");
            match best_match {
                MatchResult::Complete(search_result) => {
                    if let Err(e) = process_search_results(vec![search_result.clone()]).await {
                        error!("Failed to process best result: {}", e);
                    }
                }
                MatchResult::Partial {
                    result: _,
                    missing_params,
                } => {
                    warn!(
                        "Best match is partial with missing parameters: {:?}",
                        missing_params
                    );
                }
            }
        }

        Ok(results)
    }
}
