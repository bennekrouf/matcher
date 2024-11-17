use anyhow::Result as AnyhowResult;
use chrono::Utc;
use iggy::clients::client::IggyClient;
use iggy::messages::send_messages::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePayload {
    timestamp: String,
    action: String,
    parameters: Vec<String>,
}

pub async fn send_structured_message(
    client: &Box<IggyClient>,
    tenant: &str,
    topic: &str,
    action: &str,
    parameters: Vec<String>,
) -> AnyhowResult<()> {
    // Create the message payload
    let payload = MessagePayload {
        timestamp: Utc::now().to_rfc3339(),
        action: action.to_string(),
        parameters,
    };
    let json_payload = serde_json::to_string(&payload)?;

    // Create and initialize producer
    let mut producer = client.producer(tenant, topic)?.build();
    producer.init().await?;

    // Create and send message
    let message = Message::from_str(&json_payload)?;
    producer.send(vec![message]).await?;

    println!("Sent message: {}", json_payload);
    Ok(())
}
