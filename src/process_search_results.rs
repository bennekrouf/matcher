use crate::candle::load_model::load_model;
use crate::database::SearchResult;
use crate::messaging::get_authenticated_iggy_client::get_authenticated_iggy_client;
use crate::messaging::send_structured_message::send_structured_message;
use anyhow::{anyhow, Result as AnyhowResult};
use clap::Parser;
use lazy_static::lazy_static;

#[derive(Parser)]
#[command(
    name = "matcher",
    about = "Match natural language queries to endpoints",
    long_about = "A tool for semantically matching natural language queries to API endpoints using embeddings",
    version,
    author = "Mohamed <mb@mayorana.ch>",
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

pub async fn process_search_results(results: Vec<SearchResult>) -> AnyhowResult<()> {
    // Only proceed with the best match (first result)
    let best_match = if let Some(result) = results.first() {
        result
    } else {
        return Ok(()); // No results to process
    };

    println!(
        "Processing best match with similarity: {}",
        best_match.similarity
    );

    let client = get_authenticated_iggy_client().await?;
    let message_params: Vec<String> = best_match.parameters.values().cloned().collect();
    if let Err(e) = send_structured_message(
        &client,
        "gibro",
        "notification",
        &best_match.endpoint_id,
        message_params,
    )
    .await
    {
        eprintln!(
            "Failed to send message for endpoint {}: {}",
            best_match.endpoint_id, e
        );
        return Err(anyhow!("Message sending failed: {}", e));
    }
    println!(
        "Sent notification for endpoint: {} with similarity: {}",
        best_match.endpoint_id, best_match.similarity
    );

    println!("Completed processing best match");
    Ok(())
}
