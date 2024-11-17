use super::db::VectorDB;
use crate::config::{Config, SearchResult};
use crate::search_operations::search_similar;
use anyhow::Result as AnyhowResult;

impl VectorDB {
    pub async fn search_similar(
        &self,
        query: &str,
        language: &str,
        limit: usize,
        config: &Config,
    ) -> AnyhowResult<(Vec<SearchResult>, f32)> {
        search_similar(&self.patterns_table, query, language, limit, config).await
    }
}
