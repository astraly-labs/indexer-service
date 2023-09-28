use diesel::result::Error;
use diesel_async::pooled_connection::deadpool::PoolError;

#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("internal server error")]
    InternalServerError,
    #[error("not found")]
    NotFound,
}

impl From<Error> for InfraError {
    fn from(value: Error) -> Self {
        match value {
            Error::NotFound => InfraError::NotFound,
            _ => InfraError::InternalServerError,
        }
    }
}

impl From<PoolError> for InfraError {
    fn from(_value: PoolError) -> Self {
        Self::InternalServerError
    }
}
