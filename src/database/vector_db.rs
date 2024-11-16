use super::db_initializer::initialize_table;
use crate::config::Config;
use crate::database::SearchResult;
use crate::search_operations::search_similar;
use anyhow::Result as AnyhowResult;
use lancedb::{connect, Connection, Table};

pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    patterns_table: Table,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;

        if with_init {
            if let Some(ref config) = config {
                println!("Initializing database with endpoints from config...");

                match connection.drop_table("patterns").await {
                    Ok(_) => println!("Dropped existing patterns table."),
                    Err(e) => println!("Note: Couldn't drop table (might not exist): {}", e),
                }

                initialize_table(&connection, &config.endpoints).await?;
            }
        }

        let patterns_table = match connection.open_table("patterns").execute().await {
            Ok(table) => table,
            Err(e) => {
                if e.to_string().contains("Table not found") && config.is_some() {
                    println!("Table not found, creating new one...");
                    initialize_table(&connection, &config.unwrap().endpoints).await?;
                    connection.open_table("patterns").execute().await?
                } else {
                    return Err(e.into());
                }
            }
        };

        Ok(Self {
            connection,
            patterns_table,
        })
    }

    pub async fn search_similar(
        &self,
        query: &str,
        language: &str,
        limit: usize,
    ) -> AnyhowResult<(Vec<SearchResult>, f32)> {
        search_similar(&self.patterns_table, query, language, limit).await
    }
}
