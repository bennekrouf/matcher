use crate::filters::extract_app_name::extract_app_name;
use crate::filters::extract_email::extract_email;
use anyhow::Result as AnyhowResult;
use std::collections::HashMap;
use tracing::{debug, info};

pub fn extract_parameters(query: &str, pattern: &str) -> AnyhowResult<HashMap<String, String>> {
    debug!("Extracting parameters from query: '{}'", query);
    debug!("Using pattern: '{}'", pattern);

    let mut params = HashMap::new();

    // Check for email parameter
    if pattern.contains("{email}") {
        debug!("Pattern contains email parameter");
        match extract_email(query) {
            Some(email) => {
                info!("Found email: {:?}", email);
                params.insert("email".to_string(), email.as_str().to_string());
            }
            None => debug!("No email found in query"),
        }
    }

    // Check for app parameter
    if pattern.contains("{app}") {
        debug!("Pattern contains app parameter");
        match extract_app_name(query) {
            Some(app) => {
                info!("Found app name: {}", app);
                params.insert("app".to_string(), app);
            }
            None => debug!("No app name found in query"),
        }
    }

    debug!("Extracted parameters: {:?}", params);
    Ok(params)
}

// Let's also add a test module to verify the functionality:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_extraction() {
        let test_cases = vec![
            (
                "envoie un email à test@example.com",
                "envoyer un mail à {email}",
                vec![("email", "test@example.com")],
            ),
            (
                "envoie un email à mohamed.bennekrouf@gmail.com",
                "envoyer un mail à {email}",
                vec![("email", "mohamed.bennekrouf@gmail.com")],
            ),
            (
                "send an email to user.name+tag@domain.com",
                "envoyer un mail à {email}",
                vec![("email", "user.name+tag@domain.com")],
            ),
        ];

        for (query, pattern, expected_params) in test_cases {
            let params = extract_parameters(query, pattern).unwrap();
            for (key, value) in expected_params {
                assert_eq!(
                    params.get(key).unwrap(),
                    value,
                    "Failed to extract correct email from '{}', expected '{}', got '{:?}'",
                    query,
                    value,
                    params.get(key)
                );
            }
        }
    }

    #[test]
    fn test_combined_parameter_extraction() {
        let test_cases = vec![(
            "analyze app myapp and send results to test@example.com",
            "analyze {app} and send to {email}",
            vec![("app", "myapp"), ("email", "test@example.com")],
        )];

        for (query, pattern, expected_params) in test_cases {
            let params = extract_parameters(query, pattern).unwrap();
            for (key, value) in expected_params {
                assert_eq!(
                    params.get(key).unwrap(),
                    value,
                    "Failed to extract parameter '{}' from '{}'",
                    key,
                    query
                );
            }
        }
    }
}
