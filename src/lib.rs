mod config;
mod constants;
mod database;
mod embeddings;
mod extract_app_name;
mod grpc_server;
mod model;
mod preprocessing;
mod process_search_results;
mod send_structured_message;
#[cfg(test)]
mod tests;

// Re-export everything that main.rs needs
pub use config::Config;
pub use constants::*;
pub use database::VectorDB;
pub use grpc_server::start_grpc_server;
pub use model::load_model;
pub use process_search_results::process_search_results;
