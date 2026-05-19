#[cfg(test)]
mod tests {
    use super::*;
    use health::health_client::HealthClient;

    #[tokio::test]
    async fn test_health_check() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = HealthClient::connect("http://[::1]:50051").await?;

        let request = tonic::Request::new(HealthCheckRequest {
            service: "".to_string(),
        });

        let response = client.check(request).await?;
        assert_eq!(response.into_inner().status, ServingStatus::Serving as i32);

        Ok(())
    }
}
