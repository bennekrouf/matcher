use matcher::{
    load_model, process_search_results, start_grpc_server, Config, VectorDB, MODEL_PATH,
};

use anyhow::Result as AnyhowResult;
use clap::Parser;
use lazy_static::lazy_static;
use std::sync::Arc;

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
        if let Err(e) = start_grpc_server(config).await {
            eprintln!("Failed to start gRPC server: {}", e);
        }
    } else {
        // Run in CLI mode
        let db = VectorDB::new("data/mydb", Some(config.as_ref().clone()), args.reload).await?;
        if let Some(query) = args.query {
            println!("\nTesting vector search...");
            let (results, _similarity) = db.search_similar(&query, &args.language, 1).await?;
            process_search_results(results).await?;
        }
    }
    Ok(())
}
