use crate::config::{Endpoint, ParameterAnalysis};
use std::collections::HashMap;
use tracing::{debug, info};

impl Endpoint {
    pub fn analyze_parameters(
        &self,
        provided_params: &HashMap<String, String>,
    ) -> ParameterAnalysis {
        let mut missing_required = Vec::new();
        let mut missing_optional = Vec::new();
        let found = provided_params.clone();

        debug!("Analyzing parameters for endpoint: {}", self.id);
        debug!("Provided parameters: {:?}", provided_params);

        for param in &self.parameters {
            if !provided_params.contains_key(&param.name) {
                if param.required {
                    debug!(
                        "Missing required parameter: {} ({})",
                        param.name, param.description
                    );
                    missing_required.push(param.clone());
                } else {
                    debug!(
                        "Missing optional parameter: {} ({})",
                        param.name, param.description
                    );
                    missing_optional.push(param.clone());
                }
            } else {
                debug!(
                    "Found parameter: {} = {}",
                    param.name,
                    provided_params.get(&param.name).unwrap()
                );
            }
        }

        let analysis = ParameterAnalysis {
            missing_required,
            missing_optional,
            found,
        };

        info!(
            "Parameter analysis for endpoint '{}': {:?}",
            self.id, analysis
        );
        analysis
    }
}
