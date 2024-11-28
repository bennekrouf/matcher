use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub pattern: String,
    pub similarity: f32,
    pub parameters: HashMap<String, String>,
    pub parameter_analysis: ParameterAnalysis,
}

#[derive(Debug)]
pub struct ProcessedQuery {
    pub cleaned_text: String,
    pub parameters: HashMap<String, String>,
    pub is_negated: bool,
}

//pub struct SearchAttempt {
//    pub result: Option<SearchResult>,
//    #[allow(dead_code)]
//    pub similarity: f32, // Always include the similarity score
//}

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

#[derive(Debug, Clone)]
pub struct ParameterAnalysis {
    pub missing_required: Vec<Parameter>,
    pub missing_optional: Vec<Parameter>,
    pub found: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NegationPattern {
    pub pattern: &'static str,
    pub count: i32,
}

pub struct LanguagePatterns {
    pub negations: Vec<NegationPattern>,
    pub articles: Vec<&'static str>,
    pub polite_phrases: Vec<&'static str>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
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
