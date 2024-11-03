mod config;
mod db;
mod embeddings;
mod extract_app_name;
mod model;
mod preprocessing;
mod send_structured_message;
#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::db::{SearchResult, VectorDB};
use crate::model::load_model;
use anyhow::Result as AnyhowResult;
use iggy::client::Client;
use iggy::client::UserClient;
use iggy::clients::client::IggyClient;
use lazy_static::lazy_static;
use send_structured_message::send_structured_message;

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
    help_template = "{about}\n\nUSAGE:\n    {usage}\n\n{options}"
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
        //for result in results {
        //    println!(
        //        "Matched endpoint: {} (similarity: {:.2})",
        //        result.endpoint_id, result.similarity
        //    );
        //    println!("Pattern: {}", result.pattern);
        //    if let Some(app) = result.parameters.get("app") {
        //        println!("Application: {}", app);
        //    }
        //
        //    println!("Message sent!");
        //}
        process_search_results(results).await?;
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

async fn process_search_results(results: Vec<SearchResult>) -> AnyhowResult<()> {
    let client = IggyClient::default();
    client.connect().await?;
    client.login_user("iggy", "iggy").await?;

    for result in results {
        // Convert parameters to message format using the endpoint_id directly
        let message_params: Vec<String> = result.parameters.values().cloned().collect();

        send_structured_message(
            &client,
            "gibro",
            "notification",
            &result.endpoint_id, // Use the endpoint_id directly as the action
            message_params,
        )
        .await?;

        println!(
            "Sent notification for endpoint: {} with similarity: {}",
            result.endpoint_id, result.similarity
        );
    }

    Ok(())
}
