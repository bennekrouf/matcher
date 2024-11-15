use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
}

pub fn extract_email(text: &str) -> Option<regex::Match<'_>> {
    EMAIL_REGEX.find(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_extraction() {
        let test_cases = vec![
            ("test@example.com", Some("test@example.com")),
            ("email to test@example.com", Some("test@example.com")),
            (
                "mohamed.bennekrouf@gmail.com",
                Some("mohamed.bennekrouf@gmail.com"),
            ),
            (
                "send to user.name+tag@domain.com",
                Some("user.name+tag@domain.com"),
            ),
            ("no email here", None),
        ];

        for (input, expected) in test_cases {
            let result = extract_email(input);
            match expected {
                Some(expected_email) => {
                    assert!(result.is_some(), "Failed to extract email from: {}", input);
                    assert_eq!(
                        result.unwrap().as_str(),
                        expected_email,
                        "Extracted wrong email from: {}",
                        input
                    );
                }
                None => assert!(
                    result.is_none(),
                    "Should not have found email in: {}",
                    input
                ),
            }
        }
    }
}
