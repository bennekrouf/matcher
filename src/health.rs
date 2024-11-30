use crate::database::vector_db::VectorDB;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod health {
    tonic::include_proto!("grpc.health.v1");
}

use health::health_check_response::ServingStatus;
use health::health_server::Health;
use health::{HealthCheckRequest, HealthCheckResponse};

#[derive(Clone)]
pub struct HealthService {
    db: Arc<VectorDB>,
}

impl HealthService {
    pub fn new(db: Arc<VectorDB>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        // Check if DB is accessible
        match self.db.check_connection().await {
            Ok(_) => Ok(Response::new(HealthCheckResponse {
                status: ServingStatus::Serving as i32,
            })),
            Err(_) => Ok(Response::new(HealthCheckResponse {
                status: ServingStatus::NotServing as i32,
            })),
        }
    }
}
