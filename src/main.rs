mod config;
mod db;
mod embeddings;
mod extract_app_name;
mod grpc_server;
mod model;
mod preprocessing;
mod process_search_results;
mod send_structured_message;
#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::db::VectorDB;
use crate::model::load_model;
use anyhow::Result as AnyhowResult;
use clap::Parser;
use lazy_static::lazy_static;
use process_search_results::process_search_results;
use std::sync::Arc;

const MODEL_PATH: &str = "models/multilingual-MiniLM";
const CONFIG_PATH: &str = "endpoints.yaml";

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

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let args = Args::parse();
    println!("Loading model from: {}", MODEL_PATH);

    let config = Arc::new(Config::load_from_yaml(CONFIG_PATH)?);

    if args.server {
        // Run in server mode
        println!("Starting gRPC server...");
        if let Err(e) = grpc_server::start_grpc_server(config).await {
            eprintln!("Failed to start gRPC server: {}", e);
        }
    } else {
        // Run in CLI mode
        let db = VectorDB::new("data/mydb", Some(config.as_ref().clone()), args.reload).await?;

        if let Some(query) = args.query {
            println!("\nTesting vector search...");
            let results = db.search_similar(&query, &args.language, 1).await?;
            process_search_results(results).await?;
        }
    }

    Ok(())
}
