use crate::config::Config;
use crate::config::Endpoint;
use crate::database::vector_db::VectorDB;
use anyhow::Result as AnyhowResult;
use lancedb::Connection;

pub async fn initialize_table(connection: &Connection, endpoints: &[Endpoint]) -> AnyhowResult<()> {
    println!("Initializing table...");

    let config = Config {
        endpoints: endpoints.to_vec(),
    };

    // Force creation of new table by passing config and with_init as true
    let _ = VectorDB::new_with_connection(connection.clone(), Some(config), true).await?;

    println!("Table initialization completed successfully!");
    Ok(())
}
