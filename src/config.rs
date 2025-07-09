/*!
 * Client Configuration
 * 
 * Configuration for the HTTP client connection to lmoserver.
 */

use crate::error::{ClientError, ClientResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Client configuration for connecting to the lmoserver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Server URL (e.g., "http://localhost:3000")
    pub server_url: String,
    
    /// Request timeout
    pub timeout: Duration,
    
    /// User agent string
    pub user_agent: String,
    
    /// API key for authentication (optional)
    pub api_key: Option<String>,
    
    /// Maximum number of retries for failed requests
    pub max_retries: usize,
    
    /// Delay between retries
    pub retry_delay: Duration,
    
    /// Enable request/response logging
    pub enable_logging: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            timeout: Duration::from_secs(30),
            user_agent: format!("lmoclient/{}", env!("CARGO_PKG_VERSION")),
            api_key: None,
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            enable_logging: true,
        }
    }
}

impl ClientConfig {
    /// Create a new configuration with the specified server URL
    pub fn new<S: Into<String>>(server_url: S) -> ClientResult<Self> {
        let mut config = Self::default();
        config.server_url = server_url.into();
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> ClientResult<()> {
        // Validate server URL
        Url::parse(&self.server_url)
            .map_err(|e| ClientError::ConfigError(format!("Invalid server URL: {}", e)))?;

        // Validate timeout
        if self.timeout.as_secs() == 0 {
            return Err(ClientError::ConfigError("Timeout must be greater than 0".to_string()));
        }

        // Validate retry settings
        if self.max_retries > 10 {
            return Err(ClientError::ConfigError("Max retries cannot exceed 10".to_string()));
        }

        Ok(())
    }

    /// Build the full API URL for an endpoint
    pub fn api_url<S: AsRef<str>>(&self, endpoint: S) -> ClientResult<String> {
        let base = self.server_url.trim_end_matches('/');
        let endpoint = endpoint.as_ref().trim_start_matches('/');
        
        if endpoint.is_empty() {
            Ok(base.to_string())
        } else {
            Ok(format!("{}/{}", base, endpoint))
        }
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

    /// Enable or disable logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }
}

/// Server endpoint definitions
pub struct Endpoints;

impl Endpoints {
    pub const HEALTH: &'static str = "v1/health";
    pub const MODELS_LIST: &'static str = "v1/models";
    pub const MODELS_LIST_LOCAL: &'static str = "v1/models/local";
    pub const MODELS_LOAD: &'static str = "v1/models/load";
    pub const MODELS_UNLOAD: &'static str = "v1/models/unload";
    pub const MODELS_LOADED: &'static str = "v1/models/loaded";
    pub const MODELS_STATUS: &'static str = "v1/models/status";
    pub const MODELS_DOWNLOAD: &'static str = "v1/models/download";
    pub const MODELS_DOWNLOAD_LEGACY: &'static str = "v1/models/download/legacy";
    pub const CHAT_COMPLETIONS: &'static str = "v1/chat/completions";
    pub const CHAT_COMPLETIONS_STREAM: &'static str = "v1/chat/completions/stream";
    
    /// Get download progress SSE endpoint for a specific download ID
    pub fn download_progress_sse(download_id: &str) -> String {
        format!("v1/models/download/{}/progress", download_id)
    }
    
    /// Get download control endpoint for a specific download ID
    pub fn download_control(download_id: &str) -> String {
        format!("v1/models/download/{}/control", download_id)
    }
}

/// Server endpoint type for compatibility
pub type ServerEndpoint = &'static str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server_url, "http://localhost:3000");
    }

    #[test]
    fn test_api_url_building() {
        let config = ClientConfig::default();
        
        assert_eq!(
            config.api_url("v1/models").unwrap(),
            "http://localhost:3000/v1/models"
        );
        
        assert_eq!(
            config.api_url("/v1/health").unwrap(),
            "http://localhost:3000/v1/health"
        );
    }

    #[test]
    fn test_invalid_url() {
        let result = ClientConfig::new("not-a-url");
        assert!(result.is_err());
    }
}