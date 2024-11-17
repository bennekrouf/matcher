use super::super::schema::PATTERNS_SCHEMA;
use crate::config::Config;
use anyhow::Result as AnyhowResult;
use arrow_array::{RecordBatch, RecordBatchIterator};
use arrow_schema::Schema;
use lancedb::{connect, Connection, Table};
use std::sync::Arc;

pub struct VectorDB {
    #[allow(dead_code)]
    pub(crate) connection: Connection,
    pub(crate) patterns_table: Table,
    pub(crate) patterns_schema: Arc<Schema>,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;
        Self::new_with_connection(connection, config, with_init).await
    }

    pub async fn new_with_connection(
        connection: Connection,
        config: Option<Config>,
        with_init: bool,
    ) -> AnyhowResult<Self> {
        let patterns_table = if with_init && config.is_some() {
            // Create new table with empty batch
            println!("Creating new patterns table...");
            let empty_batch = RecordBatch::new_empty(Arc::new(PATTERNS_SCHEMA.clone()));
            let batch_iterator =
                RecordBatchIterator::new(vec![Ok(empty_batch)], Arc::new(PATTERNS_SCHEMA.clone()));

            let table = connection
                .create_table("patterns", Box::new(batch_iterator))
                .execute()
                .await?;

            // Create VectorDB instance
            let db = Self {
                connection: connection.clone(),
                patterns_table: table,
                patterns_schema: Arc::new(PATTERNS_SCHEMA.clone()),
            };

            // Initialize with patterns if config is provided
            if let Some(cfg) = config {
                println!("Initializing patterns...");
                db.add_patterns(&cfg.endpoints).await?;
            }

            db.patterns_table
        } else {
            // Try to open existing table
            match connection.open_table("patterns").execute().await {
                Ok(table) => table,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Table 'patterns' not found. Please initialize the database first using --reload"
                    ));
                }
            }
        };

        Ok(Self {
            connection,
            patterns_table,
            patterns_schema: Arc::new(PATTERNS_SCHEMA.clone()),
        })
    }
}
