use super::language_patterns::LANGUAGE_PATTERNS;
use super::EMAIL_REGEX;
use crate::config::LanguagePatterns;
use crate::config::NegationPattern;
use crate::config::ProcessedQuery;
use crate::filters::extract_app_name::extract_app_name;
use std::collections::HashMap;

pub fn preprocess_query(query: &str, language: &str) -> ProcessedQuery {
    let patterns = LANGUAGE_PATTERNS.get(language).unwrap_or_else(|| {
        LANGUAGE_PATTERNS.get("en").unwrap() // fallback to English
    });

    let is_negated = count_negations(query, &patterns.negations) % 2 != 0;
    let cleaned_text = clean_text(query, patterns);

    let mut parameters = HashMap::new();
    if let Some(email) = extract_email(&cleaned_text) {
        parameters.insert("email".to_string(), email);
    }
    if let Some(app) = extract_app_name(&cleaned_text) {
        parameters.insert("app".to_string(), app);
    }

    ProcessedQuery {
        cleaned_text,
        parameters,
        is_negated,
    }
}

fn count_negations(query: &str, patterns: &[NegationPattern]) -> i32 {
    let query = query.to_lowercase();
    let mut total_negations = 0;

    // Sort patterns by length (longest first) to catch complete phrases first
    let mut sorted_patterns = patterns.to_vec();
    sorted_patterns.sort_by(|a, b| b.pattern.len().cmp(&a.pattern.len()));

    for pattern in sorted_patterns {
        if query.contains(pattern.pattern) {
            total_negations += pattern.count;
        }
    }

    total_negations
}

fn clean_text(text: &str, patterns: &LanguagePatterns) -> String {
    let mut cleaned = text.to_lowercase();

    // Remove articles
    for article in &patterns.articles {
        cleaned = cleaned.replace(article, " ");
    }

    // Remove polite phrases
    for phrase in &patterns.polite_phrases {
        cleaned = cleaned.replace(phrase, "");
    }

    // Clean up extra spaces
    cleaned.replace("  ", " ").trim().to_string()
}

pub fn extract_email(text: &str) -> Option<String> {
    EMAIL_REGEX.find(text).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_detection() {
        let test_cases = vec![
            ("envoyer un mail", "fr", false),
            ("ne pas envoyer de mail", "fr", true),
            ("ne pas ne pas envoyer de mail", "fr", false),
            ("send email", "en", false),
            ("do not send email", "en", true),
            ("don't not send email", "en", false),
        ];

        for (input, lang, should_be_negated) in test_cases {
            let processed = preprocess_query(input, lang);
            assert_eq!(
                processed.is_negated, should_be_negated,
                "Failed for '{}' ({}): expected negated={}",
                input, lang, should_be_negated
            );
        }
    }

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
                "envoie document par mail à toto@gmail.com",
            ),
            (
                "Pourriez-vous envoyer un mail à user@example.com",
                "envoyer mail à user@example.com",
            ),
        ];

        for (input, expected) in test_cases {
            let processed = preprocess_query(input, "fr");
            assert_eq!(processed.cleaned_text, expected);
        }
    }
}
