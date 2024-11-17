use anyhow::Result as AnyhowResult;
use lancedb::connect;
use matcher::initialize_table;
use matcher::{
    parse_args, process_search_results, start_grpc_server, Config, VectorDB, CONFIG_PATH,
    MODEL_PATH,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
#[tokio::main]
async fn main() -> AnyhowResult<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = parse_args();
    println!("Loading model from: {}", MODEL_PATH);
    let config = Arc::new(Config::load_from_yaml(CONFIG_PATH)?);

    // Ensure database directory exists
    let db_path = "data/mydb";
    if args.reload {
        // Create or ensure the directory exists
        if Path::new(db_path).exists() {
            fs::remove_dir_all(db_path)?;
        }
        fs::create_dir_all(db_path)?;

        println!("Initializing/reloading database...");
        let connection = connect(db_path).execute().await?;
        initialize_table(&connection, &config.endpoints).await?;
        println!("Database initialization complete");

        if !args.server {
            return Ok(());
        }
    }

    if args.server {
        println!("Starting gRPC server...");
        if let Err(e) = start_grpc_server(config).await {
            eprintln!("Failed to start gRPC server: {}", e);
        }
    } else if !args.reload {
        // Only enter this branch if not reloading
        let db = VectorDB::new(db_path, None, false).await?;
        if let Some(query) = args.query {
            println!("\nTesting vector search...");
            let (results, _similarity) = db
                .search_similar(&query, &args.language, 1, &config)
                .await?;
            process_search_results(results).await?;
        }
    }
    Ok(())
}
