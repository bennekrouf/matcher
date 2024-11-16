mod db_initializer;
mod search_operations;
mod vector_db;

pub use vector_db::VectorDB;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
}
