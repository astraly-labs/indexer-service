use deadpool_diesel::InteractError;
use diesel_async::pooled_connection::deadpool::PoolError;

#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("internal server error")]
    InternalServerError,
    #[error("not found")]
    NotFound,
}

pub fn adapt_infra_error<T: Error>(error: T) -> InfraError {
    error.as_infra_error()
}

pub trait Error {
    fn as_infra_error(&self) -> InfraError;
}

impl Error for diesel::result::Error {
    fn as_infra_error(&self) -> InfraError {
        match self {
            diesel::result::Error::NotFound => InfraError::NotFound,
            _ => InfraError::InternalServerError,
        }
    }
}

impl Error for PoolError {
    fn as_infra_error(&self) -> InfraError {
        InfraError::InternalServerError
    }
}

impl Error for InteractError {
    fn as_infra_error(&self) -> InfraError {
        InfraError::InternalServerError
    }
}
