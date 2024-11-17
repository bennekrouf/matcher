use std::collections::HashMap;

use anyhow::{Context, Result as AnyhowResult};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{DistanceType, Table};

use super::process_search_batch::process_search_batch;
use crate::candle::get_embeddings::get_embeddings;
use crate::config::{Config, SearchResult};
use crate::preprocessing::preprocess_query::preprocess_query;
use futures::StreamExt;

pub async fn search_similar(
    patterns_table: &Table,
    query: &str,
    language: &str,
    limit: usize,
    config: &Config,
) -> AnyhowResult<(Vec<SearchResult>, f32)> {
    let processed = preprocess_query(query, language);
    println!("\nProcessed query: '{}'", processed.cleaned_text);
    let query_embedding = get_embeddings(&processed.cleaned_text).await?;
    println!("Generated query embedding, starting vector search...");
    let mut results = patterns_table
        .vector_search(query_embedding)
        .context("Failed to create vector search")?
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await?;

    let mut initial_matches = Vec::new();
    let mut best_similarity: f32 = 0.0;

    while let Some(Ok(rb)) = results.next().await {
        let (new_matches, similarity) = process_search_batch(rb, &processed, config).await?;
        best_similarity = best_similarity.max(similarity);
        initial_matches.extend(new_matches);
    }

    // Deduplicate matches by endpoint_id
    let mut best_matches: HashMap<String, SearchResult> = HashMap::new();
    for match_data in initial_matches {
        best_matches
            .entry(match_data.endpoint_id.clone())
            .and_modify(|existing| {
                if match_data.similarity > existing.similarity {
                    *existing = match_data.clone();
                }
            })
            .or_insert(match_data);
    }

    // Create final sorted results
    let mut deduplicated_matches = best_matches.into_values().collect::<Vec<_>>();
    deduplicated_matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    Ok((deduplicated_matches, best_similarity))
}
