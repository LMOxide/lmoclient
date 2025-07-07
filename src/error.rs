/*!
 * Client Error Types
 * 
 * Error handling for the HTTP client library.
 */

use thiserror::Error;

/// Result type alias for client operations
pub type ClientResult<T> = Result<T, ClientError>;

/// Client error types
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model operation failed: {0}")]
    ModelOperationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

impl ClientError {
    /// Create a server error from HTTP status and message
    pub fn from_response(status: u16, message: String) -> Self {
        match status {
            401 | 403 => Self::AuthenticationError(message),
            404 => Self::ModelNotFound(message),
            _ => Self::ServerError { status, message },
        }
    }

    /// Check if this error type is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::HttpError(e) => e.is_timeout() || e.is_connect(),
            Self::ServerError { status, .. } => matches!(status, 500..=599),
            Self::TimeoutError(_) => true,
            Self::NetworkError(_) => true,
            _ => false,
        }
    }

    /// Get the error status code if available
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::ServerError { status, .. } => Some(*status),
            Self::HttpError(e) => e.status().map(|s| s.as_u16()),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}

impl From<url::ParseError> for ClientError {
    fn from(err: url::ParseError) -> Self {
        Self::ConfigError(format!("URL parse error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let error = ClientError::ModelNotFound("test-model".to_string());
        assert!(!error.is_retryable());
        assert_eq!(error.status_code(), None);

        let error = ClientError::ServerError {
            status: 500,
            message: "Internal error".to_string(),
        };
        assert!(error.is_retryable());
        assert_eq!(error.status_code(), Some(500));
    }

    #[test]
    fn test_from_response() {
        let error = ClientError::from_response(404, "Not found".to_string());
        assert!(matches!(error, ClientError::ModelNotFound(_)));

        let error = ClientError::from_response(401, "Unauthorized".to_string());
        assert!(matches!(error, ClientError::AuthenticationError(_)));

        let error = ClientError::from_response(500, "Server error".to_string());
        assert!(matches!(error, ClientError::ServerError { .. }));
    }
}