use anyhow::{Context, Result as AnyhowResult};
use candle_core::{Device, Tensor};

use crate::process_search_results::AI;

pub async fn get_embeddings(sentence: &str) -> AnyhowResult<Vec<f32>> {
    let (model, tokenizer) = &*AI;
    let tokens = tokenizer
        .encode_batch(vec![sentence], true)
        .map_err(|e| anyhow::anyhow!("Failed to encode sentence: {}", e))?;

    let token_ids = tokens
        .iter()
        .map(|tokens| {
            let tokens = tokens.get_ids().to_vec();
            Ok(Tensor::new(tokens.as_slice(), &Device::Cpu)?)
        })
        .collect::<AnyhowResult<Vec<_>>>()
        .context("Failed to create token tensors")?;

    let token_ids = Tensor::stack(&token_ids, 0)?;
    let token_type_ids = token_ids.zeros_like()?;

    let attention_mask = tokens
        .iter()
        .map(|tokens| {
            let mask = tokens.get_attention_mask();
            Ok(Tensor::new(mask, &Device::Cpu)?)
        })
        .collect::<AnyhowResult<Vec<_>>>()
        .context("Failed to create attention masks")?;

    let attention_mask = Tensor::stack(&attention_mask, 0)?;
    let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;

    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
    let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
    let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;

    let embeddings = embeddings.to_vec2::<f32>()?;
    Ok(embeddings.into_iter().next().unwrap())
}
