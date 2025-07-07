/*!
 * LMOclient Main Client Implementation
 * 
 * HTTP client for communicating with the LMOxide server.
 */

use reqwest::{Client, Response};
use tracing::{debug, info, warn};

use crate::config::{ClientConfig, Endpoints};
use crate::error::{ClientError, ClientResult};
use crate::models::{
    ChatRequestBuilder, DownloadModelRequest, DownloadModelResponse, HealthInfo, 
    LoadModelRequest, LoadModelResponse, ModelListResponse, ModelStatusInfo, 
    UnloadModelRequest, UnloadModelResponse,
};
use crate::streaming::ChatCompletionStream;

// Re-export server types
use lmoserver::shared_types::{ChatCompletionRequest, ChatCompletionResponse, ModelInfo};

/// Main HTTP client for LMOxide server
#[derive(Debug, Clone)]
pub struct LmoClient {
    client: Client,
    config: ClientConfig,
}

impl LmoClient {
    /// Create a new client with default configuration
    pub fn new() -> ClientResult<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new client with custom server URL
    pub fn with_url<S: Into<String>>(server_url: S) -> ClientResult<Self> {
        let config = ClientConfig::new(server_url)?;
        Self::with_config(config)
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> ClientResult<Self> {
        // Validate configuration
        config.validate()?;

        // Build HTTP client
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        // Add authentication if provided
        if let Some(ref api_key) = config.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            let auth_header = format!("Bearer {}", api_key);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&auth_header)
                    .map_err(|e| ClientError::ConfigError(format!("Invalid API key: {}", e)))?,
            );
            client_builder = client_builder.default_headers(headers);
        }

        let client = client_builder
            .build()
            .map_err(|e| ClientError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Check server health
    pub async fn health(&self) -> ClientResult<HealthInfo> {
        debug!("Checking server health");
        
        let url = self.config.api_url(Endpoints::HEALTH)?;
        let response = self.make_request(reqwest::Method::GET, url, None::<&()>).await?;
        
        let health: HealthInfo = response.json().await?;
        info!("Server health check completed: {}", health.status);
        
        Ok(health)
    }

    /// List available models
    pub async fn list_models(&self) -> ClientResult<ModelListResponse> {
        debug!("Listing available models");
        
        let url = self.config.api_url(Endpoints::MODELS_LIST)?;
        let response = self.make_request(reqwest::Method::GET, url, None::<&()>).await?;
        
        // The server returns a simple array of ModelInfo, not a wrapped response
        let models: Vec<ModelInfo> = response.json().await?;
        info!("Listed {} models", models.len());
        
        // Wrap in our response structure for consistency
        let response = ModelListResponse {
            models: models.clone(),
            total: Some(models.len() as u32),
            has_more: false, // We don't have pagination info from server
        };
        
        Ok(response)
    }

    /// Load a model
    pub async fn load_model(&self, request: LoadModelRequest) -> ClientResult<LoadModelResponse> {
        info!("Loading model: {}", request.model_id);
        
        let url = self.config.api_url(Endpoints::MODELS_LOAD)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let load_response: LoadModelResponse = response.json().await?;
        
        if load_response.success {
            let duration = load_response.duration_ms.unwrap_or(0);
            let memory_mb = load_response.memory_usage_bytes
                .map(|b| b / 1024 / 1024)
                .unwrap_or(0);
            info!(
                "Model loaded successfully: {} ({}ms, {}MB)", 
                load_response.model_id,
                duration,
                memory_mb
            );
        } else {
            warn!("Model loading failed: {} - {}", 
                load_response.model_id, 
                load_response.message);
        }
        
        Ok(load_response)
    }

    /// Unload a model
    pub async fn unload_model(&self, request: UnloadModelRequest) -> ClientResult<UnloadModelResponse> {
        info!("Unloading model: {}", request.instance_id);
        
        let url = self.config.api_url(Endpoints::MODELS_UNLOAD)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let unload_response: UnloadModelResponse = response.json().await?;
        
        if unload_response.success {
            info!(
                "Model unloaded successfully: {} (freed {}MB)", 
                unload_response.model_id,
                unload_response.memory_freed_bytes / 1024 / 1024
            );
        } else {
            warn!("Model unloading failed: {} - {}", 
                unload_response.model_id, 
                unload_response.message);
        }
        
        Ok(unload_response)
    }

    /// Get model status
    pub async fn model_status(&self, model_id: &str) -> ClientResult<ModelStatusInfo> {
        debug!("Getting status for model: {}", model_id);
        
        let url = self.config.api_url(&format!("{}/{}", Endpoints::MODELS_STATUS, model_id))?;
        let response = self.make_request(reqwest::Method::GET, url, None::<&()>).await?;
        
        let status: ModelStatusInfo = response.json().await?;
        debug!("Model status: {} - {}", model_id, status.status);
        
        Ok(status)
    }

    /// Get all loaded models
    pub async fn loaded_models(&self) -> ClientResult<Vec<ModelStatusInfo>> {
        debug!("Getting loaded models");
        
        let url = self.config.api_url(Endpoints::MODELS_LOADED)?;
        let response = self.make_request(reqwest::Method::GET, url, None::<&()>).await?;
        
        let models: Vec<ModelStatusInfo> = response.json().await?;
        info!("Found {} loaded models", models.len());
        
        Ok(models)
    }

    /// Download a model from a remote repository
    pub async fn download_model(&self, request: DownloadModelRequest) -> ClientResult<DownloadModelResponse> {
        info!("Downloading model: {}", request.model_name);
        
        let url = self.config.api_url(Endpoints::MODELS_DOWNLOAD)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let download_response: DownloadModelResponse = response.json().await?;
        
        if download_response.success {
            let size_mb = download_response.size_bytes
                .map(|b| b / 1024 / 1024)
                .unwrap_or(0);
            let duration = download_response.duration_ms.unwrap_or(0);
            info!(
                "Model downloaded successfully: {} ({}ms, {}MB)", 
                download_response.model_name,
                duration,
                size_mb
            );
        } else {
            warn!("Model download failed: {} - {}", 
                download_response.model_name, 
                download_response.message);
        }
        
        Ok(download_response)
    }

    /// Create a chat completion (non-streaming)
    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> ClientResult<ChatCompletionResponse> {
        debug!("Creating chat completion for model: {}", request.model);
        
        let url = self.config.api_url(Endpoints::CHAT_COMPLETIONS)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let completion: ChatCompletionResponse = response.json().await?;
        info!("Chat completion created with {} choices", completion.choices.len());
        
        Ok(completion)
    }

    /// Create a streaming chat completion
    pub async fn chat_completion_stream(&self, request: ChatCompletionRequest) -> ClientResult<ChatCompletionStream> {
        debug!("Creating streaming chat completion for model: {}", request.model);
        
        // Ensure streaming is enabled in request
        let mut stream_request = request;
        stream_request.stream = Some(true);
        
        let url = self.config.api_url(Endpoints::CHAT_COMPLETIONS_STREAM)?;
        let response = self.make_request_stream(reqwest::Method::POST, url, Some(&stream_request)).await?;
        
        Ok(ChatCompletionStream::new(response))
    }

    /// Create a chat request builder
    pub fn chat(&self) -> ChatRequestBuilder {
        ChatRequestBuilder::new()
    }

    /// Make a JSON HTTP request with error handling and retries
    async fn make_request<T: serde::Serialize, U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
        body: Option<&T>,
    ) -> ClientResult<Response> {
        let mut retries = 0;
        
        loop {
            let mut request_builder = self.client.request(method.clone(), url.as_str());
            
            // Add JSON body if provided
            if let Some(body) = body {
                request_builder = request_builder.json(body);
            }
            
            // Log request if enabled
            if self.config.enable_logging {
                debug!("Making {} request to: {}", method, url.as_str());
            }
            
            // Execute request
            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if self.config.enable_logging {
                        debug!("Response status: {}", status);
                    }
                    
                    if status.is_success() {
                        return Ok(response);
                    } else {
                        // Handle error response
                        let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        let error = ClientError::from_response(status.as_u16(), error_body);
                        
                        if error.is_retryable() && retries < self.config.max_retries {
                            warn!("Retryable error (attempt {}): {}", retries + 1, error);
                            retries += 1;
                            tokio::time::sleep(self.config.retry_delay).await;
                            continue;
                        } else {
                            return Err(error);
                        }
                    }
                }
                Err(e) => {
                    let error = ClientError::HttpError(e);
                    
                    if error.is_retryable() && retries < self.config.max_retries {
                        warn!("Retryable error (attempt {}): {}", retries + 1, error);
                        retries += 1;
                        tokio::time::sleep(self.config.retry_delay).await;
                        continue;
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Make a streaming HTTP request
    async fn make_request_stream<T: serde::Serialize, U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
        body: Option<&T>,
    ) -> ClientResult<Response> {
        let mut request_builder = self.client.request(method.clone(), url.as_str());
        
        // Add JSON body if provided
        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }
        
        // Add streaming headers
        request_builder = request_builder
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache");
        
        if self.config.enable_logging {
            debug!("Making streaming {} request to: {}", method, url.as_str());
        }
        
        let response = request_builder.send().await?;
        let status = response.status();
        
        if status.is_success() {
            Ok(response)
        } else {
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(ClientError::from_response(status.as_u16(), error_body))
        }
    }
}

impl Default for LmoClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LmoClient::new();
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.config().server_url, "http://localhost:3000");
    }

    #[test]
    fn test_client_with_custom_url() {
        let client = LmoClient::with_url("http://example.com:8080");
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.config().server_url, "http://example.com:8080");
    }

    #[test]
    fn test_client_with_invalid_url() {
        let client = LmoClient::with_url("not-a-valid-url");
        assert!(client.is_err());
    }
}