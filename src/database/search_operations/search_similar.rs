use anyhow::{Context, Result as AnyhowResult};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{DistanceType, Table};

use super::process_results::process_search_batch;
use crate::database::SearchResult;
use crate::embeddings::get_embeddings;
use crate::preprocessing::preprocess_query;
use futures::StreamExt;

pub async fn search_similar(
    patterns_table: &Table,
    query: &str,
    language: &str,
    limit: usize,
) -> AnyhowResult<Vec<SearchResult>> {
    let processed = preprocess_query(query, language);
    println!("\nProcessed query: '{}'", processed.cleaned_text);

    for (param_name, param_value) in &processed.parameters {
        println!("!!! Detected {}: {}", param_name, param_value);
    }

    let query_embedding = get_embeddings(&processed.cleaned_text).await?;
    println!("Generated query embedding, starting vector search...");

    let mut results = patterns_table
        .vector_search(query_embedding)
        .context("Failed to create vector search")?
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await?;

    println!("Vector search completed, processing results...");
    let mut matches = Vec::new();

    while let Some(Ok(rb)) = results.next().await {
        matches.extend(process_search_batch(rb, &processed).await?);
    }

    println!("Final matches count: {}", matches.len());
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    Ok(matches)
}
