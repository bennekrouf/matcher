#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::matcher_service::MatcherEngine;
    use crate::vector_db::search_result::MatchResult;
    use anyhow::Result as AnyhowResult;
    use std::sync::Arc;

    //const TEST_DB_PATH: &str = "data/test_db";
    const TEST_CONFIG_PATH: &str = "endpoints.yaml";

    async fn setup() -> AnyhowResult<MatcherEngine> {
        let config = Config::load_from_yaml(TEST_CONFIG_PATH)?;
        let engine = MatcherEngine::new(Arc::new(config)).await?;
        Ok(engine)
    }

    fn get_text_and_similarity(match_result: &MatchResult) -> (&str, f32) {
        match match_result {
            MatchResult::Complete(result) => (&result.text, result.similarity),
            MatchResult::Partial { result, .. } => (&result.text, result.similarity),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        async fn setup() -> AnyhowResult<MatcherEngine> {
            let config = Config::load_from_yaml(TEST_CONFIG_PATH)?;
            MatcherEngine::new(Arc::new(config)).await
        }

        #[tokio::test]
        async fn test_endpoint_matching() -> AnyhowResult<()> {
            let engine = setup().await?;
            let test_cases = vec![
                ("analyser gpecs", "analyze_specific_repository", 0.8),
                ("lance analyse de gpecs", "analyze_specific_repository", 0.8),
                ("envoie un email à test@example.com", "send_email", 0.8),
            ];

            for (query, expected_endpoint, min_similarity) in test_cases {
                let results = engine.find_matches(query, "fr", 1).await?;
                assert!(!results.is_empty(), "No results found for query: {}", query);

                match &results[0] {
                    MatchResult::Complete(result) => {
                        assert_eq!(
                            result.endpoint_id, expected_endpoint,
                            "Wrong endpoint for query: {}",
                            query
                        );
                        assert!(
                            result.similarity >= min_similarity,
                            "Low confidence match for '{}'. Got similarity {}",
                            query,
                            result.similarity
                        );
                    }
                    MatchResult::Partial {
                        result,
                        missing_params,
                    } => {
                        println!(
                            "Got partial match for '{}' with missing params: {:?}",
                            query, missing_params
                        );
                        assert_eq!(
                            result.endpoint_id, expected_endpoint,
                            "Wrong endpoint for query: {}",
                            query
                        );
                    }
                }
            }
            Ok(())
        }

        #[tokio::test]
        async fn test_partial_matches() -> AnyhowResult<()> {
            let engine = setup().await?;

            let test_cases = vec![
                ("envoyer email", "send_email", vec!["email"]),
                ("analyser", "analyze_specific_repository", vec!["app"]),
            ];

            for (query, expected_endpoint, expected_missing) in test_cases {
                let results = engine.find_matches(query, "fr", 1).await?;
                assert!(!results.is_empty(), "No results found for query: {}", query);

                match &results[0] {
                    MatchResult::Partial {
                        result,
                        missing_params,
                    } => {
                        assert_eq!(
                            result.endpoint_id, expected_endpoint,
                            "Wrong endpoint for query: {}",
                            query
                        );
                        assert_eq!(
                            missing_params, &expected_missing,
                            "Wrong missing parameters for query: {}",
                            query
                        );
                    }
                    MatchResult::Complete(_) => {
                        panic!("Expected partial match for query: {}", query);
                    }
                }
            }
            Ok(())
        }

        #[tokio::test]
        async fn test_complete_matches() -> AnyhowResult<()> {
            let engine = setup().await?;

            let test_cases = vec![
                (
                    "envoyer un email à test@example.com",
                    "send_email",
                    vec![("email", "test@example.com")],
                ),
                (
                    "analyser gpecs",
                    "analyze_specific_repository",
                    vec![("app", "gpecs")],
                ),
            ];

            for (query, expected_endpoint, expected_params) in test_cases {
                let results = engine.find_matches(query, "fr", 1).await?;
                assert!(!results.is_empty(), "No results found for query: {}", query);

                match &results[0] {
                    MatchResult::Complete(result) => {
                        assert_eq!(
                            result.endpoint_id, expected_endpoint,
                            "Wrong endpoint for query: {}",
                            query
                        );
                        for (key, value) in expected_params {
                            assert_eq!(
                                result.parameters.get(key).unwrap(),
                                value,
                                "Wrong parameter value for {} in query: {}",
                                key,
                                query
                            );
                        }
                    }
                    MatchResult::Partial { .. } => {
                        panic!("Expected complete match for query: {}", query);
                    }
                }
            }
            Ok(())
        }
    }
}
