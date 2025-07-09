/*!
 * Client Model Types
 * 
 * Model types used by the HTTP client for requests and responses.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Re-export server types for convenience
pub use lmoserver::shared_types::{ChatCompletionRequest, ChatCompletionResponse, ModelInfo};

/// Response wrapper for model list operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelListResponse {
    pub models: Vec<ModelInfo>,
    pub total: Option<u32>,
    pub has_more: bool,
}

/// Information about a locally downloaded/cached model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalModelInfo {
    /// Path to the model file
    pub path: PathBuf,
    /// Filename of the model
    pub filename: String,
    /// Size of the model file in bytes
    pub size_bytes: u64,
    /// When the file was last modified
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Extracted metadata (if available)
    pub metadata: Option<serde_json::Value>,
    /// Whether this model file is currently loaded in memory
    pub is_loaded: bool,
}

/// Response wrapper for local model list operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalModelsResponse {
    pub success: bool,
    pub models: Vec<LocalModelInfo>,
    pub total_count: usize,
    pub total_size_bytes: u64,
}

/// Health check information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthInfo {
    pub status: String,
    pub timestamp: String,
    pub server_version: String,
    pub uptime_seconds: u64,
}

/// Load model request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadModelRequest {
    pub model_id: String,
    pub filename: Option<String>,
    pub config: Option<LoadModelConfig>,
}

/// Load model configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LoadModelConfig {
    pub max_memory_gb: Option<f32>,
    pub gpu_layers: Option<u32>,
    pub context_size: Option<u32>,
    pub force_reload: bool,
}

/// Load model response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadModelResponse {
    pub success: bool,
    pub message: String,
    pub model_id: String,
    pub instance_id: Option<String>,
    pub status: Option<serde_json::Value>, // ModelStatus from server
    pub duration_ms: Option<u64>,
    pub memory_usage_bytes: Option<u64>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Unload model request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnloadModelRequest {
    pub instance_id: String,
}

/// Unload model response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnloadModelResponse {
    pub success: bool,
    pub message: String,
    pub model_id: String,
    pub instance_id: String,
    pub memory_freed_bytes: u64,
    pub duration_ms: u64,
}

/// Model status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelStatusInfo {
    pub instance_id: String,
    pub model_id: String,
    pub status: String,
    pub memory_usage_bytes: u64,
    pub loaded_at: String,
}

/// Download model request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadModelRequest {
    pub model_name: String,
    pub format_hint: Option<String>,
    pub force_redownload: bool,
    pub custom_directory: Option<String>,
}

/// Download model response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadModelResponse {
    pub success: bool,
    pub message: String,
    pub model_name: String,
    pub model_id: Option<String>,
    pub download_path: Option<String>,
    pub detected_format: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
    pub error_details: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

// Re-export SSE download types from server
pub use lmoserver::download::{
    StartDownloadResponse, DownloadControlRequest, DownloadControlResponse,
    DownloadEvent, DownloadEventType, DownloadState, DownloadProgress,
    DownloadId
};

/// Chat request builder for convenient API usage
pub struct ChatRequestBuilder {
    request: ChatCompletionRequest,
}

impl ChatRequestBuilder {
    pub fn new() -> Self {
        Self {
            request: ChatCompletionRequest {
                model: String::new(),
                messages: vec![],
                temperature: None,
                top_p: None,
                n: None,
                stream: None,
                stop: None,
                max_tokens: None,
                presence_penalty: None,
                frequency_penalty: None,
                logit_bias: None,
                seed: None,
                user: None,
            },
        }
    }

    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.request.model = model.into();
        self
    }

    pub fn message<S: Into<String>>(mut self, role: S, content: S) -> Self {
        self.request.messages.push(lmoserver::shared_types::ChatMessage {
            role: role.into(),
            content: content.into(),
            name: None,
        });
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.request.max_tokens = Some(max_tokens);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.request.temperature = Some(temperature);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = Some(stream);
        self
    }

    pub fn build(self) -> ChatCompletionRequest {
        self.request
    }
}

impl Default for ChatRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}