//! Core error types for Rengo

use thiserror::Error;

/// Core error type for Rengo operations
#[derive(Debug, Error)]
pub enum RengoError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Storage error: {source}")]
    Storage {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Consensus error: {source}")]
    Consensus {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Query error: {message}")]
    Query { message: String },

    #[error("Transaction error: {message}")]
    Transaction { message: String },

    #[error("Security error: {message}")]
    Security { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Index error: {message}")]
    Index { message: String },

    #[error("Parse error: {message}")]
    Parse { message: String },
}

impl RengoError {
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config { message: message.into() }
    }

    pub fn storage<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        Self::Storage { source: Box::new(error) }
    }

    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network { message: message.into() }
    }

    pub fn consensus<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        Self::Consensus { source: Box::new(error) }
    }

    pub fn query<S: Into<String>>(message: S) -> Self {
        Self::Query { message: message.into() }
    }

    pub fn transaction<S: Into<String>>(message: S) -> Self {
        Self::Transaction { message: message.into() }
    }

    pub fn security<S: Into<String>>(message: S) -> Self {
        Self::Security { message: message.into() }
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal { message: message.into() }
    }

    pub fn index<S: Into<String>>(message: S) -> Self {
        Self::Index { message: message.into() }
    }

    pub fn parse<S: Into<String>>(message: S) -> Self {
        Self::Parse { message: message.into() }
    }
}

/// Result type alias for Rengo operations
pub type Result<T> = std::result::Result<T, RengoError>;

impl From<uuid::Error> for RengoError {
    fn from(error: uuid::Error) -> Self {
        RengoError::internal(format!("UUID parsing error: {}", error))
    }
}
