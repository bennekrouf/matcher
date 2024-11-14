use crate::db::SearchResult;
use crate::model::load_model;
use crate::send_structured_message::send_structured_message;
//use anyhow::Result as AnyhowResult;
use anyhow::{anyhow, Result as AnyhowResult};
use clap::Parser;
use iggy::client::Client;
use iggy::client::UserClient;
use iggy::clients::builder::IggyClientBuilder;
use lazy_static::lazy_static;

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

    let client = match IggyClientBuilder::new()
        .with_tcp()
        .with_server_address("iggy.mayorana.ch:8090".to_string())
        .build()
    {
        Ok(client) => {
            println!("Successfully built Iggy client");
            client
        }
        Err(e) => {
            eprintln!("Failed to build Iggy client: {}", e);
            return Err(anyhow!("Failed to build Iggy client: {}", e));
        }
    };

    match client.connect().await {
        Ok(_) => println!("Successfully connected to Iggy server"),
        Err(e) => {
            eprintln!("Failed to connect to Iggy server: {}", e);
            eprintln!("This could be due to network issues or server being unreachable");
            return Err(anyhow!("Connection failed: {}", e));
        }
    }

    if let Err(e) = client.login_user("iggy", "iggy").await {
        eprintln!("Failed to login to Iggy: {}", e);
        return Err(anyhow!("Login failed: {}", e));
    }
    println!("Logged in to Iggy");

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
