use async_trait::async_trait;
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[async_trait]
pub trait Service {
    /// Starts the service, usually a start script
    async fn start(&self);

    /// Stop the service, usually a stop script
    async fn stop(&self);

    /// Whether service is currently being used
    async fn busy(&self) -> ServiceResult<bool>;

    /// Whether service is actually running
    async fn running(&self) -> ServiceResult<bool>;
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("external error: {0}")]
    External(Box<dyn std::error::Error + Send + Sync>),
}

pub mod minecraft;
