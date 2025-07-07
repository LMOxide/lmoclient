/*!
 * LMOclient Error Types
 * 
 * Comprehensive error handling for HTTP client operations.
 */

use thiserror::Error;

/// Result type for client operations
pub type ClientResult<T> = Result<T, ClientError>;

/// Client error types
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Server returned error {status}: {message}")]
    ServerError {
        status: u16,
        message: String,
    },

    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model already loaded: {0}")]
    ModelAlreadyLoaded(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal client error: {0}")]
    InternalError(String),
}

impl ClientError {
    /// Create a server error from HTTP response
    pub fn from_response(status: u16, body: String) -> Self {
        ClientError::ServerError {
            status,
            message: body,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            ClientError::HttpError(reqwest_err) => {
                // Network errors are generally retryable
                reqwest_err.is_connect() || reqwest_err.is_timeout()
            }
            ClientError::ServerError { status, .. } => {
                // 5xx server errors are retryable, 4xx client errors are not
                *status >= 500
            }
            ClientError::ConnectionError(_) => true,
            ClientError::TimeoutError(_) => true,
            _ => false,
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            ClientError::HttpError(_) => "http",
            ClientError::JsonError(_) => "serialization",
            ClientError::UrlError(_) => "url",
            ClientError::ServerError { .. } => "server",
            ClientError::ConnectionError(_) => "connection",
            ClientError::AuthenticationError(_) => "auth",
            ClientError::TimeoutError(_) => "timeout",
            ClientError::ModelNotFound(_) => "model_not_found",
            ClientError::ModelAlreadyLoaded(_) => "model_conflict",
            ClientError::InvalidRequest(_) => "invalid_request",
            ClientError::StreamingError(_) => "streaming",
            ClientError::ConfigError(_) => "config",
            ClientError::InternalError(_) => "internal",
        }
    }
}