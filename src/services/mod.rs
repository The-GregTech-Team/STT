use async_trait::async_trait;
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[async_trait]
pub trait Service {
    async fn start(&self);
    async fn stop(&self);
    async fn busy(&self) -> ServiceResult<bool>;
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("external error: {0}")]
    External(Box<dyn std::error::Error + Send + Sync>),
}

pub mod minecraft;
