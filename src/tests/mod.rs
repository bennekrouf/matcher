#[cfg(test)]
mod tests {
    // use super::*;
    use crate::{config::Config, VectorDB};
    use anyhow::Result as AnyhowResult;
    // use pretty_assertions::assert_eq;

    const TEST_DB_PATH: &str = "data/test_db";
    const TEST_CONFIG_PATH: &str = "endpoints.yaml";

    async fn setup() -> AnyhowResult<VectorDB> {
        let config = Config::load_from_yaml(TEST_CONFIG_PATH)?;
        VectorDB::new(TEST_DB_PATH, Some(config), false).await
    }

    #[tokio::test]
    async fn test_endpoint_matching() -> AnyhowResult<()> {
        let db = setup().await?;

        let test_cases = vec![
            ("Run an analysis", "run analysis", 0.8),
            ("Please run the analysis", "run analysis", 0.8), // Should now score higher
            ("Could you run the analysis", "run analysis", 0.8),
            ("Perform a calculation", "perform calculation", 0.8),
            ("Can you do a calculation", "perform calculation", 0.7),
        ];

        for (query, expected_match, min_similarity) in test_cases {
            let (results, _similarity) = db.search_similar(query, "en", 1).await?; // Destructure here
            assert!(!results.is_empty(), "No results found for query: {}", query);
            let best_match = &results[0];
            assert!(
            best_match.similarity >= min_similarity,
            "Low confidence match for '{}'. Expected '{}' with similarity >= {}, got '{}' with {}",
            query,
            expected_match,
            min_similarity,
            best_match.pattern,
            best_match.similarity
        );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_similar_endpoints() -> AnyhowResult<()> {
        let db = setup().await?;
        let (results, _similarity) = db.search_similar("Run computation", "fr", 2).await?; // Destructure here
        assert!(results.len() >= 2, "Expected at least 2 results");
        // Both "run analysis" and "perform calculation" should be relatively good matches
        assert!(
            results
                .iter()
                .any(|r| r.pattern == "run analysis" && r.similarity > 0.6),
            "Expected 'run analysis' to be a decent match"
        );
        assert!(
            results
                .iter()
                .any(|r| r.pattern == "perform calculation" && r.similarity > 0.6),
            "Expected 'perform calculation' to be a decent match"
        );

        Ok(())
    }
}
