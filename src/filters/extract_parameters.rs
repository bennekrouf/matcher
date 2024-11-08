use std::collections::HashMap;
use anyhow::Result as AnyhowResult;

use crate::filters::extract_app_name::extract_app_name;
use crate::filters::extract_email::extract_email;

// const VECTOR_SIZE: i32 = 384;

pub fn extract_parameters(query: &str, pattern: &str) -> AnyhowResult<HashMap<String, String>> {
    let mut params = HashMap::new();

    // Check for email parameter
    if pattern.contains("{email}") {
        if let Some(email) = extract_email(query) {
            params.insert("email".to_string(), email.as_str().to_string());
        }
    }

    // Check for app parameter
    if pattern.contains("{app}") {
        if let Some(app) = extract_app_name(query) {
            params.insert("app".to_string(), app);
        }
    }

    Ok(params)
}
