use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
    pub text: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum MatchResult {
    Complete(SearchResult),
    Partial {
        result: SearchResult,
        missing_params: Vec<String>,
    },
}

impl MatchResult {
    pub fn similarity(&self) -> f32 {
        match self {
            MatchResult::Complete(result) => result.similarity,
            MatchResult::Partial { result, .. } => result.similarity,
        }
    }

    pub fn search_result(&self) -> &SearchResult {
        match self {
            MatchResult::Complete(result) => result,
            MatchResult::Partial { result, .. } => result,
        }
    }
}
