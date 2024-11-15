use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub endpoints: Vec<Endpoint>,
}

impl Endpoint {
    // Helper method to validate patterns
    pub fn validate(&self) -> Result<(), String> {
        // Check if we have at least one pattern
        if self.patterns.is_empty() {
            return Err(format!("Endpoint {} has no patterns defined", self.id));
        }

        // Validate each pattern contains parameter placeholders if parameters are defined
        for param in &self.parameters {
            if param.required {
                let placeholder = format!("{{{}}}", param.name);
                if !self.patterns.iter().any(|p| p.contains(&placeholder)) {
                    return Err(format!(
                        "Required parameter {} not found in any pattern for endpoint {}",
                        param.name, self.id
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let f = std::fs::File::open(path)?;
        let config: Config = serde_yaml::from_reader(f)?;

        // Validate all endpoints
        for endpoint in &config.endpoints {
            endpoint.validate().map_err(|e| anyhow::anyhow!(e))?;
        }

        Ok(config)
    }
}
