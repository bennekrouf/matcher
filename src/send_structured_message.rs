use anyhow::Result;
use chrono::Utc;
use iggy::clients::client::IggyClient;
use iggy::messages::send_messages::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;
use crate::config::Parameter;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePayload {
    timestamp: String,
    action: String,
    parameters: Vec<Parameter>,
    text: String,
    description: String,
}

pub async fn send_structured_message(
    client: &IggyClient,
    tenant: &str,
    topic: &str,
    action: &str,
    text: &str,         // Add text parameter
    description: &str,  // Add description parameter
    parameters: Vec<Parameter>,
) -> Result<()> {
    let payload = MessagePayload {
        timestamp: Utc::now().to_rfc3339(),
        action: action.to_string(),
        text: text.to_string(),
        description: description.to_string(),
        parameters,
    };

    let json_payload = serde_json::to_string(&payload)?;
    let mut producer = client.producer(tenant, topic)?.build();
    producer.init().await?;

    let message = Message::from_str(&json_payload)?;
    producer.send(vec![message]).await?;

    info!(
        tenant = tenant,
        topic = topic,
        action = action,
        payload = %json_payload,
        "Successfully sent message"
    );

    Ok(())
}
