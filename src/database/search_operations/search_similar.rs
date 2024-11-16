use anyhow::{Context, Result as AnyhowResult};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{DistanceType, Table};

use super::process_search_batch::process_search_batch;
use crate::database::SearchResult;
use crate::embeddings::get_embeddings;
use crate::preprocessing::preprocess_query;
use futures::StreamExt;

pub async fn search_similar(
    patterns_table: &Table,
    query: &str,
    language: &str,
    limit: usize,
) -> AnyhowResult<(Vec<SearchResult>, f32)> {
    // Return both results and best score
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

    let mut matches = Vec::new();
    let mut best_similarity: f32 = 0.0;

    while let Some(Ok(rb)) = results.next().await {
        let (new_matches, similarity) = process_search_batch(rb, &processed).await?;
        best_similarity = best_similarity.max(similarity);
        matches.extend(new_matches);
    }

    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    Ok((matches, best_similarity))
}
