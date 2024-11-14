use crate::config::Parameter;
use crate::db::SearchResult;
use crate::model::load_model;
use crate::send_structured_message::send_structured_message;
use anyhow::{anyhow, Result as AnyhowResult};
use clap::Parser;
use iggy::client::Client;
use iggy::client::UserClient;
use iggy::clients::builder::IggyClientBuilder;
use lazy_static::lazy_static;
use tracing::{error, info};

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
    let best_match = if let Some(result) = results.first() {
        result
    } else {
        return Ok(());
    };

    info!(
        "Processing best match with similarity: {}",
        best_match.similarity
    );

    let client = match IggyClientBuilder::new()
        .with_tcp()
        .with_server_address("iggy.mayorana.ch:8090".to_string())
        .build()
    {
        Ok(client) => {
            info!("Successfully built Iggy client");
            client
        }
        Err(e) => {
            error!("Failed to build Iggy client: {}", e);
            return Err(anyhow!("Failed to build Iggy client: {}", e));
        }
    };

    match client.connect().await {
        Ok(_) => info!("Connected to Iggy server"),
        Err(e) => {
            error!("Failed to connect to Iggy server: {}", e);
            return Err(anyhow!("Connection failed: {}", e));
        }
    }

    if let Err(e) = client.login_user("iggy", "iggy").await {
        error!("Failed to login to Iggy: {}", e);
        return Err(anyhow!("Login failed: {}", e));
    }
    info!("Logged in to Iggy");

    let message_params: Vec<Parameter> = best_match
        .parameters
        .iter()
        .map(|(name, value)| Parameter {
            name: name.clone(),
            value: Some(value.clone()),
            description: None,
            required: true,
        })
        .collect();

    send_structured_message(
        &client,
        "gibro",
        "notification",
        &best_match.endpoint_id,
        &best_match.text,
        &best_match.description,
        message_params,
    )
    .await?;

    info!(
        "Sent notification for endpoint: {} with similarity: {} and pattern: {}",
        best_match.endpoint_id, best_match.similarity, best_match.pattern
    );

    Ok(())
}
