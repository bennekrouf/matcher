use anyhow::Result as AnyhowResult;
use candle_core::Device;
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use std::path::Path;
use tokenizers::PaddingParams;
use tokenizers::Tokenizer;

use super::MODEL_PATH;

pub fn load_model() -> AnyhowResult<(BertModel, Tokenizer)> {
    let model_path = Path::new(MODEL_PATH);

    let config_path = model_path.join("config.json");
    let tokenizer_path = model_path.join("tokenizer.json");
    let weights_path = model_path.join("model.ot");

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Config file not found at {:?}",
            config_path
        ));
    }
    if !tokenizer_path.exists() {
        return Err(anyhow::anyhow!(
            "Tokenizer file not found at {:?}",
            tokenizer_path
        ));
    }
    if !weights_path.exists() {
        return Err(anyhow::anyhow!(
            "Model weights not found at {:?}",
            weights_path
        ));
    }

    let config = std::fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&config)?;

    let mut tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

    let vb = VarBuilder::from_pth(&weights_path, DTYPE, &Device::Cpu)?;
    let model = BertModel::load(vb, &config)?;

    if let Some(pp) = tokenizer.get_padding_mut() {
        pp.strategy = tokenizers::PaddingStrategy::BatchLongest;
    } else {
        let pp = PaddingParams {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(pp));
    }

    Ok((model, tokenizer))
}
