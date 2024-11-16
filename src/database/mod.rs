mod db_initializer;
mod vector_db;

pub use vector_db::VectorDB;

use crate::search_operations::parameter_analysis::ParameterAnalysis;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
    pub parameter_analysis: Option<ParameterAnalysis>,
}
