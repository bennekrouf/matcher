use crate::config::{Config, Endpoint};
use anyhow::Result as AnyhowResult;
use lancedb::{connect, Connection, Table};
use std::collections::HashMap;

pub struct VectorDB {
    #[allow(dead_code)]
    connection: Connection,
    pub patterns_table: Table,
    pub endpoints: HashMap<String, Endpoint>,
}

impl VectorDB {
    pub async fn new(db_path: &str, config: Option<Config>, with_init: bool) -> AnyhowResult<Self> {
        let connection = connect(db_path).execute().await?;

        if with_init && config.is_some() {
            Self::handle_initialization(&connection, config.as_ref().unwrap()).await?;
        }

        let patterns_table = Self::get_or_create_table(&connection, config.as_ref()).await?;
        let endpoints = Self::build_endpoints_map(config.as_ref());

        Ok(Self {
            connection,
            patterns_table,
            endpoints,
        })
    }

    fn build_endpoints_map(config: Option<&Config>) -> HashMap<String, Endpoint> {
        config.map_or(HashMap::new(), |c| {
            c.endpoints
                .iter()
                .map(|e| (e.id.clone(), e.clone()))
                .collect()
        })
    }
}
