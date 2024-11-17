use crate::preprocessing::APP_PATTERNS;

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

