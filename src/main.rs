mod model;
mod embeddings;
mod db;
mod config;
mod preprocessing;
#[cfg(test)]
mod tests;
mod extract_app_name;

use anyhow::Result as AnyhowResult;
use lazy_static::lazy_static;
use crate::model::load_model;
use crate::db::VectorDB;
use crate::config::Config;

// const MODEL_PATH: &str = "models/all-MiniLM-L6-v2";
const MODEL_PATH: &str = "models/multilingual-MiniLM";
const CONFIG_PATH: &str = "endpoints.yaml";

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "matcher",
    about = "Match natural language queries to endpoints",
    long_about = "A tool for semantically matching natural language queries to API endpoints using embeddings",
    version,
    author = "Your Name <your.email@example.com>",
    help_template = "{about}\n\nUSAGE:\n    {usage}\n\n{options}",
)]
struct Args {
    /// Reload the database with current YAML config
    #[arg(long, default_value = "false")]
    reload: bool,

    /// Query to test
    #[arg(short, long)]
    query: Option<String>,

    /// Show debug information
    #[arg(long)]
    debug: bool,

    /// Show all matches
    #[arg(long)]
    all: bool,

    /// Language for the query (en, fr, etc)
    #[arg(short, long, default_value = "fr")]
    language: String,
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
    let config = Config::load_from_yaml(CONFIG_PATH)?;
    let db = VectorDB::new("data/mydb", Some(config), args.reload).await?;

    println!("\nTesting vector search...");

    if let Some(query) = args.query {
        let results = db.search_similar(&query, &args.language, 1).await?;
        // println!("Results returned : {:?}", results);
        for result in results {
            println!(
                "Matched text: {} (similarity: {:.2})", 
                result.endpoint_id, 
                result.similarity
            );
            println!("Pattern: {}", result.pattern);
            if let Some(app) = result.parameters.get("app") {
                println!("Application: {}", app);
            }
        }
    }

    // Filter results above certain threshold
    // let _filtered = results.iter()
        // .filter(|r| r.similarity > 0.5)
        // .collect::<Vec<_>>();

    // Find best match
    // if let Some(best_match) = results.iter().max_by(|a, b| {
    //     a.similarity.partial_cmp(&b.similarity).unwrap()
    // }) {
    //     println!("Best match: {} ({:.2})", best_match.text, best_match.similarity);
    // }

    Ok(())
}
