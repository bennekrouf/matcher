use iggy::clients::builder::IggyClientBuilder;
use anyhow::Result as AnyhowResult;
use clap::Parser;
use lazy_static::lazy_static;
use crate::send_structured_message::send_structured_message;
use crate::config::Parameter;
use crate::db::SearchResult;
use crate::model::load_model;
use iggy::client::UserClient;
use iggy::client::Client;
use tracing::info;

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
    let client = IggyClientBuilder::new()
        .with_tcp()
        .with_server_address("abjad.mayorana.ch:8090".to_string())
        .build()?;
    client.connect().await?;
    client.login_user("iggy", "iggy").await?;

    for result in results {
        info!(
            "Processing result for endpoint {} with similarity {}",
            result.endpoint_id,
            result.similarity
        );

        let message_params: Vec<Parameter> = result
            .parameters
            .into_iter()
            .map(|(name, value)| {
                info!("Parameter: {} = {}", name, value);
                Parameter {
                    name,
                    value: Some(value),
                    description: None,
                    required: true,
                }
            })
            .collect();

        send_structured_message(
            &client,
            "gibro",
            "notification",
            &result.endpoint_id,
            &result.text,         // Pass the text
            &result.description,  // Pass the description
            message_params,
        )
        .await?;

        println!(
            "Sent notification for endpoint: {} with similarity: {} and pattern: {}",
            result.endpoint_id, 
            result.similarity,
            result.pattern
        );
    }

    Ok(())
}
