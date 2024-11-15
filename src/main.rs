mod cli;
mod config;
mod constants;
mod embeddings;
mod filters;
mod grpc;
mod load_model;
mod matcher_service;
mod preprocessing;
mod process_search_results;
mod send_structured_message;

#[cfg(test)]
mod tests;
mod vector_db;

use crate::config::Config;
use crate::load_model::load_model;
use crate::vector_db::db::VectorDB;
use anyhow::Result as AnyhowResult;
use clap::Parser;
use grpc::start_grpc_server::start_grpc_server;
use lazy_static::lazy_static;
use process_search_results::process_search_results;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vector_db::SearchResult;

#[derive(Parser)]
#[command(
    name = "matcher",
    about = "Match natural language queries to endpoints",
    long_about = "A tool for semantically matching natural language queries to API endpoints using embeddings",
    version,
    author = "Your Name <your.email@example.com>",
    help_template = "{about}\n\nUSAGE:\n    {usage}\n\n{options}"
)]
struct Args {
    #[arg(long, default_value = "false")]
    reload: bool,
    #[arg(short, long)]
    query: Option<String>,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    all: bool,
    #[arg(short, long, default_value = "fr")]
    language: String,
    #[arg(long)]
    server: bool,
}

lazy_static! {
    pub(crate) static ref AI: (
        candle_transformers::models::bert::BertModel,
        tokenizers::Tokenizer
    ) = load_model().expect("Unable to load model");
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug,your_crate_name=debug"));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    setup_logging();
    let args = cli::parse_args();
    let (model_path, config_path) = cli::get_paths();
    if args.debug {
        println!("Debug mode enabled");
        println!("Loading model from: {}", model_path);
    }
    let config = Arc::new(Config::load_from_yaml(config_path)?);

    match (args.server, args.query) {
        (true, _) => {
            println!("Starting gRPC server...");
            if let Err(e) = start_grpc_server(config).await {
                eprintln!("Failed to start gRPC server: {}", e);
                std::process::exit(1);
            }
        }
        (false, Some(query)) => {
            let db = VectorDB::new("data/mydb", Some(config.as_ref().clone()), args.reload).await?;
            println!("\nExecuting vector search...");
            let match_results = db
                .search_similar(&query, &args.language, if args.all { 5 } else { 1 })
                .await?;

            // Convert MatchResult to SearchResult
            let search_results: Vec<SearchResult> = match_results
                .into_iter()
                .map(|m| m.search_result().clone())
                .collect();
            process_search_results(search_results).await?;
        }
        (false, None) => {
            eprintln!("Error: Either --server or --query must be specified");
            eprintln!("For help, run: matcher --help");
            std::process::exit(1);
        }
    }
    Ok(())
}
