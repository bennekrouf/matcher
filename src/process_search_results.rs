use crate::database::SearchResult;
use crate::messaging::get_authenticated_iggy_client::get_authenticated_iggy_client;
use crate::messaging::send_structured_message::send_structured_message;
use anyhow::{anyhow, Result as AnyhowResult};

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
