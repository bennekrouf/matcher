use crate::{filters::extract_app_name::extract_app_name, preprocessing::EMAIL_REGEX};
use anyhow::Result as AnyhowResult;
use std::collections::HashMap;

pub fn extract_parameters(query: &str, pattern: &str) -> AnyhowResult<HashMap<String, String>> {
    let mut params = HashMap::new();

    if pattern.contains("{email}") {
        if let Some(email) = EMAIL_REGEX.find(query) {
            params.insert("email".to_string(), email.as_str().to_string());
        }
    }

    if pattern.contains("{app}") {
        if let Some(app) = extract_app_name(query) {
            params.insert("app".to_string(), app);
        }
    }

    Ok(params)
}
