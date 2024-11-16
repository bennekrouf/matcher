use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NegationPattern {
    pub pattern: &'static str,
    pub count: i32,
}

lazy_static! {
    pub static ref LANGUAGE_PATTERNS: HashMap<&'static str, LanguagePatterns> = {
        let mut m = HashMap::new();

        // French patterns
        m.insert("fr", LanguagePatterns {
            negations: vec![
                NegationPattern { pattern: "ne pas", count: 1 },
                NegationPattern { pattern: "pas", count: 1 },
                NegationPattern { pattern: "ne", count: 1 },
                NegationPattern { pattern: "non", count: 1 },
                NegationPattern { pattern: "ne pas ne pas", count: 2 },
                NegationPattern { pattern: "pas ne pas", count: 2 },
            ],
            articles: vec![" le ", " la ", " les ", " l'"],
            polite_phrases: vec![
                "s'il vous plaît ",
                "s'il vous plait ",
                "pourriez-vous ",
                "pouvez-vous ",
                "je voudrais ",
                "je souhaite ",
            ],
        });

        // English patterns
        m.insert("en", LanguagePatterns {
            negations: vec![
                NegationPattern { pattern: "do not", count: 1 },
                NegationPattern { pattern: "don't", count: 1 },
                NegationPattern { pattern: "not", count: 1 },
                NegationPattern { pattern: "never", count: 1 },
                NegationPattern { pattern: "don't not", count: 2 },
                NegationPattern { pattern: "do not not", count: 2 },
                NegationPattern { pattern: "never not", count: 2 },
            ],
            articles: vec![" the "],
            polite_phrases: vec![
                "please ",
                "could you ",
                "can you ",
                "would you ",
            ],
        });

        m
    };
}

pub struct LanguagePatterns {
    pub negations: Vec<NegationPattern>,
    pub articles: Vec<&'static str>,
    pub polite_phrases: Vec<&'static str>,
}
