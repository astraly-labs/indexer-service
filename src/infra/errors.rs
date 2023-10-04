use diesel::result::Error;
use diesel_async::pooled_connection::deadpool::PoolError;
use strum::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("internal server error: {0}")]
    InternalServerError(Error),
    #[error("not found")]
    NotFound,
    #[error("pool error: {0}")]
    PoolError(PoolError),
    #[error("parsing failed: {0}")]
    ParseError(ParseError),
}

impl From<Error> for InfraError {
    fn from(value: Error) -> Self {
        match value {
            Error::NotFound => InfraError::NotFound,
            _ => InfraError::InternalServerError(value),
        }
    }
}

impl From<PoolError> for InfraError {
    fn from(value: PoolError) -> Self {
        Self::PoolError(value)
    }
}
