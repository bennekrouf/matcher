use crate::{
    config::SearchResult,
    grpc::matcher_service::matcher::{EndpointMatch, ParameterInfo},
};

pub fn create_endpoint_match(
    result: &SearchResult,
    is_negated: bool,
    similarity: f32,
) -> EndpointMatch {
    EndpointMatch {
        endpoint_id: result.endpoint_id.clone(),
        similarity: similarity as f64,
        parameters: result.parameters.clone(),
        is_negated,
        missing_required: result
            .parameter_analysis
            .missing_required
            .iter()
            .map(|p| ParameterInfo {
                name: p.name.clone(),
                description: p.description.clone(),
                required: true,
            })
            .collect(),
        missing_optional: result
            .parameter_analysis
            .missing_optional
            .iter()
            .map(|p| ParameterInfo {
                name: p.name.clone(),
                description: p.description.clone(),
                required: false,
            })
            .collect(),
    }
}
