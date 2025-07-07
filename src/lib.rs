/*!
 * LMOclient - HTTP Client Library for LMOxide Server
 * 
 * A comprehensive HTTP client for communicating with the LMOxide server.
 * Provides high-level abstractions for model management, chat completions,
 * and server administration.
 */

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod streaming;

// Re-export main types for convenience
pub use client::LmoClient;
pub use config::{ClientConfig, ServerEndpoint};
pub use error::{ClientError, ClientResult};

// Re-export server types for external use
pub use lmoserver::shared_types::{ModelInfo, ChatMessage, ChatCompletionRequest, ChatCompletionResponse, ModelSpecifier, PredictionConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_creation() {
        let config = ClientConfig::default();
        assert_eq!(config.server_url, "http://localhost:3000");
        assert!(config.timeout.as_secs() > 0);
    }
}