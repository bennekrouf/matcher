use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub variations: Vec<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub endpoints: Vec<Endpoint>,
}

impl Config {
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let f = std::fs::File::open(path)?;
        Ok(serde_yaml::from_reader(f)?)
    }
}

impl Endpoint {
    // Helper to get all text representations
    pub fn all_texts(&self) -> Vec<String> {
        let mut texts = vec![self.text.clone()];
        texts.extend(self.variations.clone());
        texts
    }
}
