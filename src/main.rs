mod cli;
mod config;
mod db;
mod embeddings;
mod filters;
mod model;
mod preprocessing;
mod send_structured_message;
mod grpc_server;
#[cfg(test)]
mod tests;
mod process_search_results;
mod constants;

use std::sync::Arc;
use anyhow::Result as AnyhowResult;
use lazy_static::lazy_static;
use process_search_results::process_search_results;
use tracing_subscriber;
use crate::db::VectorDB;
use crate::config::Config;
use crate::model::load_model;

lazy_static! {
    pub(crate) static ref AI: (
        candle_transformers::models::bert::BertModel,
        tokenizers::Tokenizer
    ) = load_model().expect("Unable to load model");
}

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info,your_crate_name=debug")
        });
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
            if let Err(e) = grpc_server::start_grpc_server(config).await {
                eprintln!("Failed to start gRPC server: {}", e);
                std::process::exit(1);
            }
        }
        (false, Some(query)) => {
            let db = VectorDB::new("data/mydb", Some(config.as_ref().clone()), args.reload).await?;
            println!("\nExecuting vector search...");
            let results = db.search_similar(&query, &args.language, if args.all { 5 } else { 1 }).await?;
            process_search_results(results).await?;
        }
        (false, None) => {
            eprintln!("Error: Either --server or --query must be specified");
            eprintln!("For help, run: matcher --help");
            std::process::exit(1);
        }
    }

    Ok(())
}
