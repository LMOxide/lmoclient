/*!
 * LMOclient Model Types
 * 
 * Client-specific model types and request/response structures.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export server types for convenience
pub use lmoserver::shared_types::{ModelInfo, ChatMessage, ChatCompletionRequest, ChatCompletionResponse, ModelSpecifier, PredictionConfig};
pub use lmoserver::models::model_management::{LoadModelRequest as ServerLoadModelRequest, UnloadModelRequest as ServerUnloadModelRequest};

/// Client-side model list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    /// List of available models
    pub models: Vec<ModelInfo>,
    /// Total count (may be more than returned if limited)
    pub total: Option<u32>,
    /// Whether there are more results
    pub has_more: bool,
}

/// Model loading request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelRequest {
    /// Model identifier to load
    pub model_id: String,
    /// Specific download option (filename) to load
    pub filename: Option<String>,
    /// Force reload if already loaded
    pub force_reload: Option<bool>,
}

/// Model loading response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelResponse {
    /// Success status
    pub success: bool,
    /// Model ID that was loaded
    pub model_id: String,
    /// Loading time in milliseconds
    pub load_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Any warnings during loading
    pub warnings: Vec<String>,
}

/// Model unloading request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelRequest {
    /// Instance ID of the loaded model to unload
    pub instance_id: String,
    /// Force unload even if in use
    pub force: Option<bool>,
}

/// Model unloading response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelResponse {
    /// Success status
    pub success: bool,
    /// Model ID that was unloaded
    pub model_id: String,
    /// Memory freed in bytes
    pub memory_freed_bytes: u64,
}

/// Model status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatusInfo {
    /// Model ID
    pub model_id: String,
    /// Current status
    pub status: String,
    /// Load time
    pub loaded_at: Option<u64>,
    /// Memory usage
    pub memory_usage_bytes: u64,
    /// Number of requests processed
    pub requests_processed: u64,
    /// Average inference time
    pub avg_inference_time_ms: f64,
}

/// Server health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Overall status
    pub status: String,
    /// Server version
    pub version: Option<String>,
    /// Uptime in seconds
    pub uptime_seconds: Option<u64>,
    /// Available backends
    pub backends: Option<HashMap<String, serde_json::Value>>,
    /// Loaded models count
    pub loaded_models: Option<u32>,
    /// Memory usage information
    pub memory: Option<MemoryInfo>,
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Memory used by models in bytes
    pub model_memory_bytes: u64,
}

/// Chat completion request builder
#[derive(Debug, Clone)]
pub struct ChatRequestBuilder {
    messages: Vec<ChatMessage>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stream: bool,
    stop: Vec<String>,
}

impl ChatRequestBuilder {
    /// Create a new chat request builder
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            model: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: false,
            stop: Vec::new(),
        }
    }

    /// Add a message to the conversation
    pub fn message<R: Into<String>, C: Into<String>>(mut self, role: R, content: C) -> Self {
        self.messages.push(ChatMessage {
            role: role.into(),
            content: content.into(),
            name: None,
        });
        self
    }

    /// Add a user message
    pub fn user<S: Into<String>>(self, content: S) -> Self {
        self.message("user", content)
    }

    /// Add an assistant message
    pub fn assistant<S: Into<String>>(self, content: S) -> Self {
        self.message("assistant", content)
    }

    /// Add a system message
    pub fn system<S: Into<String>>(self, content: S) -> Self {
        self.message("system", content)
    }

    /// Set the model to use
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set maximum tokens to generate
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature for sampling
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_p for nucleus sampling
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Enable streaming response
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Add stop sequence
    pub fn stop<S: Into<String>>(mut self, stop: S) -> Self {
        self.stop.push(stop.into());
        self
    }

    /// Build the chat completion request
    pub fn build(self) -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: self.model.unwrap_or_else(|| "default".to_string()),
            messages: self.messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            stream: Some(self.stream),
            stop: if self.stop.is_empty() { None } else { Some(self.stop) },
            user: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            seed: None,
            n: None,
        }
    }
}

impl Default for ChatRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequestBuilder::new()
            .system("You are a helpful assistant")
            .user("Hello, how are you?")
            .model("gpt-3.5-turbo")
            .max_tokens(100)
            .temperature(0.7)
            .build();

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }
}