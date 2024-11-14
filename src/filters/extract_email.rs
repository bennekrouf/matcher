
use crate::filters::custom_regex::EMAIL_REGEX;

pub fn extract_email(text: &str) -> Option<String> {
    EMAIL_REGEX.find(text)
        .map(|m| m.as_str().to_string())
}
