pub mod language_patterns;
pub mod preprocess_query;

use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;
#[derive(Debug)]
pub struct ProcessedQuery {
    pub cleaned_text: String,
    pub parameters: HashMap<String, String>,
    pub is_negated: bool,
}

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    // Pattern for finding app names in French sentences
    static ref APP_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        ("de ", ""), // "analyse de gpecs"
        ("du ", ""), // "analyse du gpecs"
        ("pour ", ""), // "analyse pour gpecs"
        ("sur ", ""), // "analyse sur gpecs"
        (" de l'application ", ""), // "analyse de l'application gpecs"
        (" de l'app ", ""), // "analyse de l'app gpecs"
    ];
}
