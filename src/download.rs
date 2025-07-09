/*!
 * Download Progress with Server-Sent Events
 * 
 * Provides real-time download progress tracking via SSE streaming.
 * Supports starting, monitoring, and controlling downloads.
 */

use futures::stream::Stream;
use reqwest;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

use crate::config::Endpoints;
use crate::error::{ClientError, ClientResult};
use crate::models::{
    DownloadControlRequest, DownloadControlResponse, DownloadEvent, DownloadId,
    DownloadModelRequest, StartDownloadResponse,
};
use crate::client::LmoClient;

/// Parsed SSE event types
#[derive(Debug)]
enum ParsedSseEvent {
    /// Download progress event with JSON data
    DownloadEvent(String),
    /// Keep-alive event from Axum
    KeepAlive,
    /// Heartbeat event from server
    Heartbeat,
}

/// Download progress stream using Server-Sent Events
pub struct DownloadProgressStream {
    sse_url: String,
    download_id: DownloadId,
}

impl DownloadProgressStream {
    /// Create a new download progress stream
    pub fn new(sse_url: String, download_id: DownloadId) -> ClientResult<Self> {
        Ok(Self {
            sse_url,
            download_id,
        })
    }

    /// Get the download ID
    pub fn download_id(&self) -> &DownloadId {
        &self.download_id
    }

    /// Convert to a stream of download events using a basic SSE implementation
    pub fn into_stream(self) -> impl Stream<Item = ClientResult<DownloadEvent>> {
        let sse_url = self.sse_url.clone();
        
        async_stream::stream! {
            // Create HTTP client for SSE with timeout
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120)) // 2 minute timeout
                .build()
                .map_err(|e| ClientError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
            
            // Make SSE request
            let response = match client
                .get(&sse_url)
                .header("Accept", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    yield Err(ClientError::HttpError(e));
                    return;
                }
            };
            
            // Stream the response bytes
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            
            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Convert bytes to string and add to buffer
                        let chunk_str = match String::from_utf8(chunk.to_vec()) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Invalid UTF-8 in SSE stream: {}", e);
                                continue;
                            }
                        };
                        
                        buffer.push_str(&chunk_str);
                        
                        // Process complete SSE events (ending with \n\n)
                        while let Some(event_end) = buffer.find("\n\n") {
                            let event_data = buffer[..event_end].to_string();
                            buffer.drain(..event_end + 2);
                            
                            debug!("Raw SSE event data: {:?}", event_data);
                            
                            // Parse SSE event
                            if let Some(parsed_event) = Self::parse_sse_event(&event_data) {
                                match parsed_event {
                                    ParsedSseEvent::DownloadEvent(json_data) => {
                                        match serde_json::from_str::<DownloadEvent>(&json_data) {
                                            Ok(download_event) => yield Ok(download_event),
                                            Err(e) => {
                                                error!("Failed to parse download event JSON: {}", e);
                                                yield Err(ClientError::JsonParseError(e));
                                            }
                                        }
                                    }
                                    ParsedSseEvent::KeepAlive => {
                                        // Keep-alive event received, don't yield anything but continue the stream
                                        debug!("Received keep-alive event");
                                    }
                                    ParsedSseEvent::Heartbeat => {
                                        // Heartbeat event received, don't yield anything but continue the stream
                                        debug!("Received heartbeat event");
                                    }
                                }
                            } else {
                                debug!("Failed to parse SSE event: {:?}", event_data);
                            }
                        }
                    }
                    Err(e) => {
                        error!("SSE stream error: {}", e);
                        // Check if this is a connection/network error vs a decode error
                        if e.to_string().contains("connection closed") || 
                           e.to_string().contains("stream ended") ||
                           e.to_string().contains("connection reset") {
                            // This is expected when download completes - break without error
                            break;
                        } else {
                            yield Err(ClientError::StreamError(format!("Stream error: {}", e)));
                            break;
                        }
                    }
                }
            }
        }
    }
    
    /// Parse SSE event format and extract structured data
    fn parse_sse_event(event_data: &str) -> Option<ParsedSseEvent> {
        let mut event_type = None;
        let mut data = None;
        let mut has_comment = false;
        
        for line in event_data.lines() {
            if let Some(event) = line.strip_prefix("event: ") {
                event_type = Some(event.to_string());
            } else if let Some(event_data) = line.strip_prefix("data: ") {
                data = Some(event_data.to_string());
            } else if line.starts_with(":") {
                // SSE comment line (used for keep-alive)
                has_comment = true;
            }
        }
        
        match (event_type.as_deref(), data.as_deref()) {
            (Some("heartbeat"), Some("ping")) => {
                debug!("Parsed heartbeat event");
                Some(ParsedSseEvent::Heartbeat)
            }
            (None, Some("keep-alive")) => {
                debug!("Parsed keep-alive event");
                Some(ParsedSseEvent::KeepAlive)
            }
            (_, Some(json_data)) if json_data.starts_with('{') && json_data.ends_with('}') => {
                debug!("Parsed download event");
                Some(ParsedSseEvent::DownloadEvent(json_data.to_string()))
            }
            // Handle empty data (keep-alive) or comment-only events
            (None, Some("")) | (None, None) if has_comment => {
                debug!("Parsed SSE comment/keep-alive event");
                Some(ParsedSseEvent::KeepAlive)
            }
            _ => {
                debug!("Unknown SSE event: event_type={:?}, data={:?}, has_comment={}", event_type, data, has_comment);
                None // Unknown or invalid event
            }
        }
    }
}

impl LmoClient {
    /// Start a download and return a download ID immediately (new async API)
    pub async fn download_start(&self, request: DownloadModelRequest) -> ClientResult<StartDownloadResponse> {
        info!("Starting async download for model: {}", request.model_name);
        
        let url = self.config().api_url(Endpoints::MODELS_DOWNLOAD)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let download_response: StartDownloadResponse = response.json().await?;
        
        info!(
            "Download started: {} -> {}",
            download_response.model_name,
            download_response.download_id
        );
        
        Ok(download_response)
    }

    /// Get a progress stream for a download using Server-Sent Events
    pub async fn download_progress_stream(&self, download_id: &DownloadId) -> ClientResult<DownloadProgressStream> {
        let sse_endpoint = Endpoints::download_progress_sse(download_id);
        let sse_url = self.config().api_url(&sse_endpoint)?;
        
        debug!("Creating SSE stream for download {} at {}", download_id, sse_url);
        
        DownloadProgressStream::new(sse_url, download_id.clone())
    }

    /// Control a download (pause, resume, cancel)
    pub async fn download_control(
        &self,
        download_id: &DownloadId,
        action: &str,
    ) -> ClientResult<DownloadControlResponse> {
        info!("Controlling download {}: {}", download_id, action);
        
        let control_endpoint = Endpoints::download_control(download_id);
        let url = self.config().api_url(&control_endpoint)?;
        
        let request = DownloadControlRequest {
            action: action.to_string(),
        };
        
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        let control_response: DownloadControlResponse = response.json().await?;
        
        if control_response.success {
            info!(
                "Download control successful: {} - {}",
                download_id,
                control_response.message
            );
        } else {
            warn!(
                "Download control failed: {} - {}",
                download_id,
                control_response.message
            );
        }
        
        Ok(control_response)
    }

    /// Pause a download
    pub async fn download_pause(&self, download_id: &DownloadId) -> ClientResult<DownloadControlResponse> {
        self.download_control(download_id, "pause").await
    }

    /// Resume a download
    pub async fn download_resume(&self, download_id: &DownloadId) -> ClientResult<DownloadControlResponse> {
        self.download_control(download_id, "resume").await
    }

    /// Cancel a download
    pub async fn download_cancel(&self, download_id: &DownloadId) -> ClientResult<DownloadControlResponse> {
        self.download_control(download_id, "cancel").await
    }

    /// Legacy synchronous download (uses the /download/legacy endpoint)
    pub async fn download_model_legacy(&self, request: DownloadModelRequest) -> ClientResult<crate::models::DownloadModelResponse> {
        info!("Downloading model (legacy): {}", request.model_name);
        
        let url = self.config().api_url(Endpoints::MODELS_DOWNLOAD_LEGACY)?;
        let response = self.make_request(reqwest::Method::POST, url, Some(&request)).await?;
        
        let download_response: crate::models::DownloadModelResponse = response.json().await?;
        
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress_stream_creation() {
        let stream = DownloadProgressStream::new(
            "http://localhost:3000/v1/models/download/test-123/progress".to_string(),
            "test-123".to_string()
        );
        
        assert!(stream.is_ok());
        let stream = stream.unwrap();
        assert_eq!(stream.download_id(), "test-123");
    }
}