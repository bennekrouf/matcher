use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub endpoints: Vec<Endpoint>,
}

// impl Config {
//     pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
//         let f = std::fs::File::open(path)?;
//         Ok(serde_yaml::from_reader(f)?)
//     }
// }

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

    // Helper to check if a pattern requires a specific parameter
    // pub fn has_parameter(&self, name: &str) -> bool {
    //     self.parameters.iter().any(|p| p.name == name)
    // }

    // Helper to get parameter details
    // pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
    //     self.parameters.iter().find(|p| p.name == name)
    // }
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
