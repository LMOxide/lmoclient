/*!
 * LMOclient Configuration
 * 
 * Configuration management for the HTTP client.
 */

use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::error::{ClientError, ClientResult};

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Server base URL
    pub server_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// API key for authentication (optional)
    pub api_key: Option<String>,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Enable request/response logging
    pub enable_logging: bool,
    /// User agent string
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            timeout: Duration::from_secs(30),
            api_key: None,
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            enable_logging: false,
            user_agent: format!("lmoclient/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new<S: Into<String>>(server_url: S) -> ClientResult<Self> {
        let server_url = server_url.into();
        
        // Validate URL
        Url::parse(&server_url)?;
        
        Ok(Self {
            server_url,
            ..Default::default()
        })
    }

    /// Set API key for authentication
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable request/response logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Set retry configuration
    pub fn with_retries(mut self, max_retries: u32, delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = delay;
        self
    }

    /// Get base URL
    pub fn base_url(&self) -> ClientResult<Url> {
        Url::parse(&self.server_url)
            .map_err(|e| ClientError::ConfigError(format!("Invalid server URL: {}", e)))
    }

    /// Get API endpoint URL
    pub fn api_url(&self, endpoint: &str) -> ClientResult<Url> {
        let mut url = self.base_url()?;
        
        // Ensure endpoint starts with /
        let endpoint = if endpoint.starts_with('/') {
            endpoint
        } else {
            &format!("/{}", endpoint)
        };
        
        // Add v1 prefix if not present
        let endpoint = if endpoint.starts_with("/v1/") {
            endpoint.to_string()
        } else {
            format!("/v1{}", endpoint)
        };
        
        url.set_path(&endpoint);
        Ok(url)
    }

    /// Validate configuration
    pub fn validate(&self) -> ClientResult<()> {
        // Validate URL
        self.base_url()?;
        
        // Validate timeout
        if self.timeout.as_millis() == 0 {
            return Err(ClientError::ConfigError("Timeout must be greater than 0".to_string()));
        }
        
        // Validate retries
        if self.max_retries > 10 {
            return Err(ClientError::ConfigError("Max retries should not exceed 10".to_string()));
        }
        
        Ok(())
    }
}

/// Server endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Description
    pub description: Option<String>,
}

impl ServerEndpoint {
    /// Create a new endpoint
    pub fn new<P: Into<String>, M: Into<String>>(path: P, method: M) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            description: None,
        }
    }

    /// Add description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Well-known API endpoints
pub struct Endpoints;

impl Endpoints {
    // Model management endpoints
    pub const MODELS_LIST: &'static str = "/models";
    pub const MODELS_LOAD: &'static str = "/models/load";
    pub const MODELS_UNLOAD: &'static str = "/models/unload";
    pub const MODELS_STATUS: &'static str = "/models/status";
    pub const MODELS_LOADED: &'static str = "/models/loaded";
    
    // Chat endpoints
    pub const CHAT_COMPLETIONS: &'static str = "/chat/completions";
    pub const CHAT_COMPLETIONS_STREAM: &'static str = "/chat/completions/stream";
    
    // Inference endpoints
    pub const COMPLETIONS: &'static str = "/completions";
    pub const EMBEDDINGS: &'static str = "/embeddings";
    
    // Server endpoints
    pub const HEALTH: &'static str = "/health";
    pub const SERVER_STATUS: &'static str = "/server/status";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ClientConfig::new("http://localhost:3000").unwrap();
        assert_eq!(config.server_url, "http://localhost:3000");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_api_url_generation() {
        let config = ClientConfig::default();
        
        let url = config.api_url("/models").unwrap();
        assert_eq!(url.path(), "/v1/models");
        
        let url = config.api_url("models").unwrap();
        assert_eq!(url.path(), "/v1/models");
        
        let url = config.api_url("/v1/models").unwrap();
        assert_eq!(url.path(), "/v1/models");
    }

    #[test]
    fn test_invalid_url() {
        let result = ClientConfig::new("not-a-valid-url");
        assert!(result.is_err());
    }
}