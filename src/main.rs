mod model;
mod embeddings;
mod db;

use anyhow::Result as AnyhowResult;
use lazy_static::lazy_static;
use crate::model::load_model;
use crate::db::VectorDB;

const MODEL_PATH: &str = "models/all-MiniLM-L6-v2";

lazy_static! {
    pub(crate) static ref AI: (
        candle_transformers::models::bert::BertModel,
        tokenizers::Tokenizer
    ) = load_model().expect("Unable to load model");
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    println!("Loading model from: {}", MODEL_PATH);

    let db = VectorDB::new("data/mydb", false).await?;

    println!("\nTesting vector search...");
    db.search_similar("run analysis", 1).await?;
    db.search_similar("perform calculation", 2).await?;

    Ok(())
}
