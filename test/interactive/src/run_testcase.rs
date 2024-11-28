use colored::*;
use futures::StreamExt;
use matcher::matcher_client::MatcherClient;
use matcher::{ConfirmationResponse, InitialQuery, InteractiveRequest, ParameterValue};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tonic::Request;

pub mod matcher {
    tonic::include_proto!("matcher");
}

pub struct TestCase {
    pub query: String,
    pub language: String,
    pub description: String,
    pub confirm: bool,
    pub parameter_values: Vec<String>,
}

pub async fn run_test_case(
    host: &str,
    test_case: TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(50));
    println!("{}: {}", "Testing".blue(), test_case.description);
    println!("Query: {}", test_case.query);
    println!("Language: {}", test_case.language);
    println!("{}", "-".repeat(50));

    let mut client = MatcherClient::connect(host.to_string()).await?;
    let start_time = Instant::now();

    // Create channel for sending requests
    let (tx_req, rx_req) = mpsc::channel(32);
    let tx_req_clone = tx_req.clone();

    // Create the streaming request
    let request = Request::new(tokio_stream::wrappers::ReceiverStream::new(rx_req));
    let response = client.interactive_match(request).await?;
    let mut response_stream = response.into_inner();

    // Send initial query
    let initial_request = InteractiveRequest {
        request: Some(matcher::interactive_request::Request::InitialQuery(
            InitialQuery {
                query: test_case.query.clone(),
                language: test_case.language,
            },
        )),
    };

    println!("📤 CLIENT: Sending initial query: {}", &test_case.query);
    tx_req.send(initial_request).await?;
    println!("✅ CLIENT: Initial query sent, waiting for server response...\n");

    let mut parameter_index = 0;

    // Handle responses
    while let Some(response) = response_stream.next().await {
        match response? {
            response if response.response.is_some() => match response.response.unwrap() {
                matcher::interactive_response::Response::ConfirmationPrompt(prompt) => {
                    println!(
                        "📥 CLIENT: Received confirmation prompt after {} seconds",
                        start_time.elapsed().as_secs()
                    );
                    //println!("\n{}", "Received confirmation prompt:".yellow());
                    println!(
                        "Endpoint: {}",
                        prompt
                            .matched_endpoint
                            .map_or("None".to_string(), |e| e.endpoint_id)
                    );

                    println!("🤔 CLIENT: Thinking before confirming...");
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    let confirmation = InteractiveRequest {
                        request: Some(matcher::interactive_request::Request::ConfirmationResponse(
                            ConfirmationResponse {
                                confirmed: test_case.confirm,
                            },
                        )),
                    };

                    println!(
                        "📤 CLIENT: Sending confirmation response: {}",
                        test_case.confirm
                    );
                    tx_req_clone.send(confirmation).await?;
                    println!("✅ CLIENT: Confirmation sent, waiting for server response...\n");
                }
                matcher::interactive_response::Response::ParameterPrompt(prompt) => {
                    println!("\n{}", "Received parameter prompt:".yellow());
                    println!(
                        "Parameter: {} ({})",
                        prompt.parameter_name,
                        if prompt.required {
                            "required"
                        } else {
                            "optional"
                        }
                    );

                    if parameter_index < test_case.parameter_values.len() {
                        let value = &test_case.parameter_values[parameter_index];
                        let parameter = InteractiveRequest {
                            request: Some(matcher::interactive_request::Request::ParameterValue(
                                ParameterValue {
                                    parameter_name: prompt.parameter_name,
                                    value: value.clone(),
                                },
                            )),
                        };

                        println!("Sending parameter value: {}", value);
                        tx_req_clone.send(parameter).await?;
                        parameter_index += 1;
                    }
                }
                matcher::interactive_response::Response::MatchResult(result) => {
                    println!("\n{}", "Received final result:".green());
                    println!("Has matches: {}", result.has_matches);
                    println!("Score: {}", result.score);
                    if !result.matches.is_empty() {
                        println!("Matched endpoint: {}", result.matches[0].endpoint_id);
                    }
                    break;
                }
            },
            _ => {
                println!("{}", "Received empty response".red());
                break;
            }
        }

        // Add small delay to make output more readable
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let elapsed = start_time.elapsed();
    println!("\n{}", format!("Test completed in {:?}", elapsed).green());
    Ok(())
}
