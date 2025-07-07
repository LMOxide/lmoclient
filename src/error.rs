/*!
 * CLI Error Types
 *
 * Error handling for CLI operations.
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Server communication failed: {0}")]
    ServerError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Command failed: {0}")]
    CommandError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Chat session error: {0}")]
    ChatError(String),
}

impl From<lmoclient::ClientError> for CliError {
    fn from(err: lmoclient::ClientError) -> Self {
        match err {
            lmoclient::ClientError::ModelNotFound(model) => CliError::ModelNotFound(model),
            lmoclient::ClientError::AuthenticationError(msg) => CliError::AuthError(msg),
            lmoclient::ClientError::ConfigError(msg) => CliError::ConfigError(msg),
            _ => CliError::ServerError(err.to_string()),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError(err.to_string())
    }
}