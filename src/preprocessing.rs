use std::collections::HashMap;
// use lazy_static::lazy_static;
// use regex::Regex;
use crate::filters::extract_app_name::extract_app_name;
use crate::filters::extract_email::extract_email;

#[derive(Debug)]
pub struct ProcessedQuery {
    pub cleaned_text: String,
    pub parameters: HashMap<String, String>,
}

//         (" de l'application ", ""), // "analyse de l'application gpecs"
//         (" de l'app ", ""), // "analyse de l'app gpecs"
//     ];
// }

pub fn preprocess_query(query: &str, language: &str) -> ProcessedQuery {
    let cleaned_text = match language {
        "fr" => preprocess_french(query),
        _ => preprocess_english(query),
    };

    let mut parameters = HashMap::new();

    // Extract email if present
    if let Some(email) = extract_email(&cleaned_text) {
        parameters.insert("email".to_string(), email);
    }

    // Extract app if present
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

    // #[test]
    // fn test_french_preprocessing() {
    //     let processed = preprocess_query("Pourriez-vous lancer l'analyse de gpecs", "fr");
    //     assert_eq!(processed.cleaned_text, "lancer analyse de gpecs");
    //     assert_eq!(processed.app_name, Some("gpecs".to_string()));
    //
    //     let processed = preprocess_query("Je voudrais effectuer le calcul pour myapp", "fr");
    //     assert_eq!(processed.cleaned_text, "effectuer calcul pour myapp");
    //     assert_eq!(processed.app_name, Some("myapp".to_string()));
    // }

    #[test]
    fn test_app_extraction() {
        let test_cases = vec![
            ("analyse de gpecs", Some("gpecs")),
            ("lancer analyse de divess", Some("divess")),
            ("analyse du siges", Some("siges")),
            ("analyse de l'application testapp", Some("testapp")),
            ("analyse de l'app myapp", Some("myapp")),
            ("analyse pour app123", Some("app123")),
            // Negative cases
            ("juste une analyse", None),
            ("analyse de ", None),
            ("analyse de l'application ", None),
            // Don't extract emails as apps
            ("analyse de user@example.com", None),
            // Don't extract multi-word strings
            ("analyse de multiple words here", None),
        ];

        for (input, expected) in test_cases {
            let result = extract_app_name(input);
            assert_eq!(
                result.as_deref(),
                expected,
                "Failed for input: '{}', got: '{:?}', expected: '{:?}'",
                input,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_french_preprocessing() {
        let test_cases = vec![
            (
                "Envoie le document par mail à toto@gmail.com",
                "envoie document par mail à toto@gmail.com"
            ),
            (
                "Pourriez-vous envoyer un mail à user@example.com",
                "envoyer mail à user@example.com"
            ),
        ];

        for (input, expected) in test_cases {
            let processed = preprocess_query(input, "fr");
            assert_eq!(processed.cleaned_text, expected);
        }
    }
}
