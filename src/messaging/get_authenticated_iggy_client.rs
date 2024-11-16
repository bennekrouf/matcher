use anyhow::{anyhow, Result as AnyhowResult};
use iggy::client::Client;
use iggy::client::UserClient;
use iggy::clients::builder::IggyClientBuilder;
use iggy::clients::client::IggyClient;
use std::boxed::Box;

pub async fn get_authenticated_iggy_client() -> AnyhowResult<Box<IggyClient>> {
    let client = match IggyClientBuilder::new()
        .with_tcp()
        .with_server_address("iggy.mayorana.ch:8090".to_string())
        .build()
    {
        Ok(client) => {
            println!("Successfully built Iggy client");
            client
        }
        Err(e) => {
            eprintln!("Failed to build Iggy client: {}", e);
            return Err(anyhow!("Failed to build Iggy client: {}", e));
        }
    };

    match client.connect().await {
        Ok(_) => {
            println!("Successfully connected to Iggy server");
        }
        Err(e) => {
            eprintln!("Failed to connect to Iggy server: {}", e);
            eprintln!("This could be due to network issues or server being unreachable");
            return Err(anyhow!("Connection failed: {}", e));
        }
    }

    if let Err(e) = client.login_user("iggy", "iggy").await {
        eprintln!("Failed to login to Iggy: {}", e);
        return Err(anyhow!("Login failed: {}", e));
    }

    println!("Logged in to Iggy");
    Ok(Box::new(client))
}
