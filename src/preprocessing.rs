use crate::filters::extract_app_name::extract_app_name;
use crate::filters::extract_email::extract_email;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ProcessedQuery {
    pub cleaned_text: String,
    pub parameters: HashMap<String, String>,
}

pub fn preprocess_query(query: &str, language: &str) -> ProcessedQuery {
    let cleaned_text = match language {
        "fr" => preprocess_french(query),
        _ => preprocess_english(query),
    };

    let mut parameters = HashMap::new();

    // Extract email if present and convert Match to String
    if let Some(email) = extract_email(&cleaned_text) {
        parameters.insert("email".to_string(), email.as_str().to_string());
    }

    // Extract app if present (already returns String)
    if let Some(app) = extract_app_name(&cleaned_text) {
        parameters.insert("app".to_string(), app);
    }

    ProcessedQuery {
        cleaned_text,
        parameters,
    }
}

fn preprocess_french(query: &str) -> String {
    query
        .to_lowercase()
        .trim()
        // Remove French articles
        .replace(" le ", " ")
        .replace(" la ", " ")
        .replace(" les ", " ")
        .replace(" l'", " ")
        // Remove French polite phrases
        .replace("s'il vous plaît ", "")
        .replace("s'il vous plait ", "")
        .replace("pourriez-vous ", "")
        .replace("pouvez-vous ", "")
        .replace("je voudrais ", "")
        .replace("je souhaite ", "")
        // Fix common contractions
        .replace("d'", "de ")
        // Clean up extra spaces
        .replace("  ", " ")
        .trim()
        .to_string()
}

fn preprocess_english(query: &str) -> String {
    query
        .to_lowercase()
        .trim()
        .replace("please ", "")
        .replace("could you ", "")
        .replace("can you ", "")
        .replace("would you ", "")
        .replace(" the ", " ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french_preprocessing() {
        let test_cases = vec![
            (
                "Envoie le document par mail à toto@gmail.com",
                "envoie document par mail à toto@gmail.com",
            ),
            (
                "Pourriez-vous envoyer un mail à user@example.com",
                "envoyer un mail à user@example.com", // Fixed: keep "un"
            ),
        ];

        for (input, expected) in test_cases {
            let processed = preprocess_query(input, "fr");
            assert_eq!(
                processed.cleaned_text, expected,
                "Preprocessing failed for '{}', got '{}', expected '{}'",
                input, processed.cleaned_text, expected
            );
        }
    }

    #[test]
    fn test_preprocessing_with_parameters() {
        let test_cases = vec![
            (
                "Envoie le document par mail à test@example.com",
                "fr",
                "envoie document par mail à test@example.com",
                vec![("email", "test@example.com")],
            ),
            (
                "Analyse de testapp s'il vous plaît",
                "fr",
                "analyse de testapp", // Fixed: remove "s'il vous plaît"
                vec![("app", "testapp")],
            ),
        ];

        for (input, lang, expected_text, expected_params) in test_cases {
            let processed = preprocess_query(input, lang);
            assert_eq!(
                processed.cleaned_text, expected_text,
                "Text mismatch for '{}': got '{}', expected '{}'",
                input, processed.cleaned_text, expected_text
            );

            for (key, value) in expected_params {
                assert_eq!(
                    processed.parameters.get(key).unwrap(),
                    value,
                    "Parameter {} mismatch for input: {}",
                    key,
                    input
                );
            }
        }
    }
}
