use anyhow::Result as AnyhowResult;
use matcher::{
    parse_args, process_search_results, start_grpc_server, Config, VectorDB, CONFIG_PATH,
    MODEL_PATH,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let args = parse_args();
    println!("Loading model from: {}", MODEL_PATH);
    let config = Arc::new(Config::load_from_yaml(CONFIG_PATH)?);

    if args.server {
        println!("Starting gRPC server...");
        if let Err(e) = start_grpc_server(config).await {
            eprintln!("Failed to start gRPC server: {}", e);
        }
    } else {
        let db = VectorDB::new("data/mydb", Some(config.as_ref().clone()), args.reload).await?;
        if let Some(query) = args.query {
            println!("\nTesting vector search...");
            let (results, _similarity) = db.search_similar(&query, &args.language, 1).await?;
            process_search_results(results).await?;
        }
    }
    Ok(())
}
