pub mod get_embeddings;
pub mod load_model;

use crate::candle::load_model::load_model;
use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref AI: (
        candle_transformers::models::bert::BertModel,
        tokenizers::Tokenizer
    ) = load_model().expect("Unable to load model");
}

pub const MODEL_PATH: &str = "models/multilingual-MiniLM";
