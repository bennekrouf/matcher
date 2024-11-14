use lazy_static::lazy_static;

lazy_static! {
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

pub fn extract_app_name(text: &str) -> Option<String> {
    let text = text.to_lowercase();

    for (prefix, suffix) in APP_PATTERNS.iter() {
        if let Some(start_pos) = text.find(prefix) {
            let start = start_pos + prefix.len();
            let remaining = &text[start..];

            // If there's a suffix, look for it
            let end_pos = if suffix.is_empty() {
                remaining.len()
            } else {
                remaining.find(suffix).unwrap_or(remaining.len())
            };

            let potential_app = remaining[..end_pos].trim();

            // Basic validation of app name
            if !potential_app.is_empty() 
                && potential_app.len() >= 2  // Minimum length
                && !potential_app.contains('@')  // Not an email
                && !potential_app.contains(' ')  // Single word
            {
                return Some(potential_app.to_string());
            }
        }
    }
    None
}

