use crate::config::Config;
use crate::database::vector_db::VectorDB;
use crate::grpc::matcher_service::matcher::interactive_response::Response::MatchResult;
use crate::grpc::matcher_service::matcher::{EndpointMatch, InteractiveResponse, MatchResponse};
use crate::interaction::state::InteractionState;
use crate::preprocessing::preprocess_query::preprocess_query;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use tracing::error;

use super::endpoint::create_endpoint_match;
use crate::grpc::matcher_service::matcher::interactive_response::Response as InteractiveResponseType;

pub async fn handle_initial_query(
    query: &str,
    language: &str,
    db: &VectorDB,
    config: &Config,
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Option<InteractionState> {
    let processed = preprocess_query(query, language);

    match db
        .search_similar(&processed.cleaned_text, language, 1, config)
        .await
    {
        Ok((results, similarity)) => {
            if let Some(result) = results.first() {
                let endpoint_match =
                    create_endpoint_match(result, processed.is_negated, similarity);

                // Send confirmation prompt
                if let Err(e) = send_confirmation_prompt(&endpoint_match, tx).await {
                    error!("Failed to send confirmation prompt: {}", e);
                    return None;
                }

                Some(InteractionState::new_awaiting_confirmation(endpoint_match))
            } else {
                if let Err(e) = send_no_matches_response(tx).await {
                    error!("Failed to send no matches response: {}", e);
                }
                None
            }
        }
        Err(e) => {
            error!("Search failed: {}", e);
            if let Err(e) = tx.send(Err(Status::internal("Search failed"))).await {
                error!("Failed to send error response: {}", e);
            }
            None
        }
    }
}

pub async fn handle_confirmation(
    confirmed: bool,
    state: InteractionState,
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Option<InteractionState> {
    match state {
        InteractionState::AwaitingConfirmation { endpoint_match } => {
            if confirmed {
                if has_missing_parameters(&endpoint_match) {
                    if let Err(e) = send_first_parameter_prompt(&endpoint_match, tx).await {
                        error!("Failed to send parameter prompt: {}", e);
                        return None;
                    }
                    Some(InteractionState::CollectingParameters {
                        endpoint_match,
                        collected_parameters: Default::default(),
                    })
                } else {
                    if let Err(e) = send_final_match_response(&endpoint_match, tx).await {
                        error!("Failed to send final match: {}", e);
                        return None;
                    }
                    Some(InteractionState::Completed { endpoint_match })
                }
            } else {
                if let Err(e) = send_cancelled_response(tx).await {
                    error!("Failed to send cancelled response: {}", e);
                }
                None
            }
        }
        _ => {
            error!("Received confirmation in invalid state");
            None
        }
    }
}

// Helper functions
async fn send_confirmation_prompt(
    endpoint_match: &EndpointMatch,
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Result<(), Status> {
    let confirmation = crate::grpc::matcher_service::matcher::ConfirmationPrompt {
        matched_endpoint: Some(endpoint_match.clone()),
    };

    tx.send(Ok(InteractiveResponse {
        response: Some(InteractiveResponseType::ConfirmationPrompt(confirmation)),
    }))
    .await
    .map_err(|e| Status::internal(format!("Failed to send confirmation prompt: {}", e)))
}

async fn send_first_parameter_prompt(
    endpoint_match: &EndpointMatch,
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Result<(), Status> {
    if let Some(first_param) = endpoint_match.missing_required.first() {
        let parameter = crate::grpc::matcher_service::matcher::ParameterPrompt {
            parameter_name: first_param.name.clone(),
            description: first_param.description.clone(),
            required: true,
            endpoint_id: endpoint_match.endpoint_id.clone(),
        };

        tx.send(Ok(InteractiveResponse {
            response: Some(InteractiveResponseType::ParameterPrompt(parameter)),
        }))
        .await
        .map_err(|e| Status::internal(format!("Failed to send parameter prompt: {}", e)))
    } else {
        Ok(())
    }
}

async fn send_final_match_response(
    endpoint_match: &EndpointMatch,
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Result<(), Status> {
    let response = MatchResponse {
        matches: vec![endpoint_match.clone()],
        score: 1.0,
        has_matches: true,
    };

    tx.send(Ok(InteractiveResponse {
        response: Some(MatchResult(response)),
    }))
    .await
    .map_err(|e| Status::internal(format!("Failed to send final match: {}", e)))
}

async fn send_cancelled_response(
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Result<(), Status> {
    let response = MatchResponse {
        matches: vec![],
        score: 0.0,
        has_matches: false,
    };

    tx.send(Ok(InteractiveResponse {
        response: Some(MatchResult(response)),
    }))
    .await
    .map_err(|e| Status::internal(format!("Failed to send cancelled response: {}", e)))
}

async fn send_no_matches_response(
    tx: &Sender<Result<InteractiveResponse, Status>>,
) -> Result<(), Status> {
    let response = MatchResponse {
        matches: vec![],
        score: 0.0,
        has_matches: false,
    };

    tx.send(Ok(InteractiveResponse {
        response: Some(MatchResult(response)),
    }))
    .await
    .map_err(|e| Status::internal(format!("Failed to send no matches response: {}", e)))
}

fn has_missing_parameters(endpoint_match: &EndpointMatch) -> bool {
    !endpoint_match.missing_required.is_empty()
}
