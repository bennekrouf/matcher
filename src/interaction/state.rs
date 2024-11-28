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
        endpoint_match: EndpointMatch, // Kept if needed for final response
    },
}

impl InteractionState {
    pub fn new_awaiting_confirmation(endpoint_match: EndpointMatch) -> Self {
        InteractionState::AwaitingConfirmation { endpoint_match }
    }

    //pub fn get_endpoint_match(&self) -> &EndpointMatch {
    //    match self {
    //        InteractionState::AwaitingConfirmation { endpoint_match } => endpoint_match,
    //        InteractionState::CollectingParameters { endpoint_match, .. } => endpoint_match,
    //        InteractionState::Completed { endpoint_match } => endpoint_match,
    //    }
    //}
}
