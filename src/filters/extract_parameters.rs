use crate::filters::extract_app_name::extract_app_name;
use anyhow::Result as AnyhowResult;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
}

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
