pub fn preprocess_query(query: &str, language: &str) -> String {
    match language {
        "fr" => preprocess_french(query),
        _ => preprocess_english(query),
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


pub fn preprocess_english(query: &str) -> String {
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
        assert_eq!(
            preprocess_french("Pourriez-vous lancer l'analyse"), 
            "lancer analyse"
        );
        assert_eq!(
            preprocess_french("Je voudrais effectuer le calcul"),
            "effectuer calcul"
        );
    }
}
