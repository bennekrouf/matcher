mod candle;
mod cli;
mod config;
mod constants;
mod database;
mod filters;
mod grpc;
mod preprocessing;
mod process_search_results;
mod send_structured_message;

#[cfg(test)]
mod tests;
// Re-export everything that main.rs needs
pub use candle::load_model::load_model;
pub use candle::MODEL_PATH;
pub use cli::parse_args;
pub use config::Config;
pub use constants::*;
pub use database::VectorDB;
pub use grpc::start_grpc_server::start_grpc_server;
pub use preprocessing::language_patterns::NegationPattern;
pub use process_search_results::process_search_results;
