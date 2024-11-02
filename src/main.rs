use std::sync::Arc;
use arrow_array::{Float32Array, RecordBatch, StringArray, FixedSizeListArray};
use arrow_array::Array;
use arrow_array::types::Float32Type;
use arrow_schema::{DataType, Field, Schema};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, DistanceType, Connection, Table};
use arrow::record_batch::RecordBatchIterator;
use futures::StreamExt;
use thiserror::Error;
use lazy_static::lazy_static;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use tokenizers::{PaddingParams, Tokenizer};
use candle_nn::VarBuilder;
use candle_core::{Device, Tensor};
use anyhow::{Context, Result as AnyhowResult};  // Renamed to avoid confusion

const VECTOR_SIZE: i32 = 384; // BERT embedding size

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Model error: {0}")]
    Model(String),
}


const MODEL_PATH: &str = "models/all-MiniLM-L6-v2";

lazy_static! {
    static ref AI: (BertModel, Tokenizer) = load_model().expect("Unable to load model");
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    println!("Loading model from: {}", MODEL_PATH);

    // Initialize database with with_init=true only the first time
    let db = VectorDB::new("data/mydb", false).await?;

    // Search example
    println!("\nTesting vector search...");
    db.search_similar("run analysis", 1).await?;
    db.search_similar("perform calculation", 2).await?;

    Ok(())
}

use std::path::Path;
fn load_model() -> AnyhowResult<(BertModel, Tokenizer)> {
    let model_path = Path::new(MODEL_PATH);

    // Load files from local directory
    let config_path = model_path.join("config.json");
    let tokenizer_path = model_path.join("tokenizer.json");
    let weights_path = model_path.join("model.ot");

    // Verify files exist
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Config file not found at {:?}", config_path));
    }
    if !tokenizer_path.exists() {
        return Err(anyhow::anyhow!("Tokenizer file not found at {:?}", tokenizer_path));
    }
    if !weights_path.exists() {
        return Err(anyhow::anyhow!("Model weights not found at {:?}", weights_path));
    }

    let config = std::fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&config)?;

    let mut tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

    let vb = VarBuilder::from_pth(&weights_path, DTYPE, &Device::Cpu)?;
    let model = BertModel::load(vb, &config)?;

    // Configure tokenizer padding
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

async fn get_embeddings(sentence: &str) -> AnyhowResult<Vec<f32>> {
    let (model, tokenizer) = &*AI;

    // Tokenize - using map_err instead of context
    let tokens = tokenizer
        .encode_batch(vec![sentence], true)
        .map_err(|e| anyhow::anyhow!("Failed to encode sentence: {}", e))?;

    // Convert to tensor
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

    // Create attention mask (1 for real tokens, 0 for padding)
    let attention_mask = tokens
        .iter()
        .map(|tokens| {
            let mask = tokens.get_attention_mask();
            Ok(Tensor::new(mask, &Device::Cpu)?)
        })
        .collect::<AnyhowResult<Vec<_>>>()
        .context("Failed to create attention masks")?;

    let attention_mask = Tensor::stack(&attention_mask, 0)?;
    // Get embeddings
    let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;

    // Average pooling and normalization
    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
    let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
    let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;

    // Convert to Vec<f32>
    let embeddings = embeddings.to_vec2::<f32>()?;
    Ok(embeddings.into_iter().next().unwrap())
}


struct VectorDB {
    connection: Connection,
    table: Table,
}

impl VectorDB {
    async fn new(db_path: &str, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;
        if with_init {

            println!("Initializing database with sample data...");
            Self::initialize_table(&connection).await?;
        }

        let table = connection.open_table("endpoints").execute().await?;

        Ok(Self {
            connection,
            table,
        })
    }

    async fn initialize_table(connection: &Connection) -> AnyhowResult<()> {
        let texts = vec![
            ("endpoint1", "run analysis"),
            ("endpoint2", "perform calculation"),
        ];

        // Generate embeddings
        println!("Generating embeddings...");
        let mut embeddings = Vec::new();
        for (_, text) in &texts {
            let embedding = get_embeddings(text).await?;
            embeddings.push(embedding);
        }

        // Define schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_SIZE,
                ),
                false,
            ),
        ]));

        // Prepare arrays
        let ids: Vec<&str> = texts.iter().map(|(id, _)| *id).collect();
        let texts_data: Vec<&str> = texts.iter().map(|(_, text)| *text).collect();

        let id_array = Arc::new(StringArray::from(ids));
        let text_array = Arc::new(StringArray::from(texts_data));
        let vector_array = Arc::new(
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                embeddings.iter().map(|vec| Some(vec.iter().copied().map(Some).collect::<Vec<_>>())),
                VECTOR_SIZE,
            ),
        );

        // Create record batch
        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![id_array, text_array, vector_array],
        )?;

        // Create table
        connection.create_table(
            "endpoints",
            Box::new(RecordBatchIterator::new(vec![Ok(record_batch)], schema)),
        )
        .execute()
        .await?;

        println!("Table created successfully!");
        Ok(())
    }

    async fn search_similar(&self, query: &str, limit: usize) -> AnyhowResult<()> {
        let query_embedding = get_embeddings(query).await?;

        let mut results = self.table
            .vector_search(query_embedding)
            .context("Failed to create vector search")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await?;

        while let Some(Ok(rb)) = results.next().await {
            let text_column = rb
                .column_by_name("text")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let distance_column = rb
                .column_by_name("_distance")
                .unwrap()
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap();

            for i in 0..text_column.len() {
                let matched_text = text_column.value(i);
                let distance = distance_column.value(i);
                let similarity = 1.0 - distance;

                println!("Matched text: {} (similarity: {:.2})", matched_text, similarity);
            }
        }
        Ok(())
    }
}
