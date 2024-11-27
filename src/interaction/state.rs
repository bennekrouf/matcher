use crate::grpc::matcher_service::matcher::EndpointMatch;
use std::collections::HashMap;

#[derive(Debug)]
pub enum InteractionState {
    AwaitingConfirmation {
        endpoint_match: EndpointMatch,
    },
    CollectingParameters {
        endpoint_match: EndpointMatch,
        collected_parameters: HashMap<String, String>,
    },
    Completed {
        endpoint_match: EndpointMatch,
    },
}

impl InteractionState {
    pub fn new_awaiting_confirmation(endpoint_match: EndpointMatch) -> Self {
        InteractionState::AwaitingConfirmation { endpoint_match }
    }

    pub fn into_collecting_parameters(self) -> Option<Self> {
        match self {
            InteractionState::AwaitingConfirmation { endpoint_match } => {
                Some(InteractionState::CollectingParameters {
                    endpoint_match,
                    collected_parameters: HashMap::new(),
                })
            }
            _ => None,
        }
    }

    pub fn into_completed(self) -> Option<Self> {
        match self {
            InteractionState::CollectingParameters { endpoint_match, .. } => {
                Some(InteractionState::Completed { endpoint_match })
            }
            _ => None,
        }
    }

    pub fn get_endpoint_match(&self) -> &EndpointMatch {
        match self {
            InteractionState::AwaitingConfirmation { endpoint_match } => endpoint_match,
            InteractionState::CollectingParameters { endpoint_match, .. } => endpoint_match,
            InteractionState::Completed { endpoint_match } => endpoint_match,
        }
    }
}
