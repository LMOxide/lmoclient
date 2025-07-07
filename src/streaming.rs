/*!
 * LMOclient Streaming Support
 * 
 * Server-Sent Events (SSE) streaming for chat completions.
 */

use std::pin::Pin;
use std::task::{Context, Poll};
use reqwest::Response;
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, warn};

use crate::error::{ClientError, ClientResult};

/// Streaming-specific types for chat completion chunks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkChoice {
    pub index: u32,
    pub delta: ChatCompletionChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

/// Individual parsed chunk from streaming response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Raw event data
    pub raw: String,
    /// Parsed streaming response (if valid JSON)
    pub chunk: Option<ChatCompletionChunk>,
    /// Whether this is the final chunk
    pub is_done: bool,
}

/// Streaming chat completion response
pub struct ChatCompletionStream {
    inner: Pin<Box<dyn Stream<Item = ClientResult<StreamChunk>> + Send>>,
}

impl ChatCompletionStream {
    /// Create a new streaming response from HTTP response
    pub fn new(response: Response) -> Self {
        let stream = response
            .bytes_stream()
            .map(|chunk| {
                chunk
                    .map_err(ClientError::HttpError)
                    .and_then(|bytes| {
                        let text = String::from_utf8_lossy(&bytes);
                        parse_sse_chunk(&text)
                    })
            });

        Self {
            inner: Box::pin(stream),
        }
    }

    /// Collect all chunks into a vector
    pub async fn collect(mut self) -> ClientResult<Vec<StreamChunk>> {
        let mut chunks = Vec::new();
        
        while let Some(chunk) = self.next().await {
            match chunk {
                Ok(chunk) => {
                    let is_done = chunk.is_done;
                    chunks.push(chunk);
                    if is_done {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(chunks)
    }

    /// Collect only the text content from all chunks
    pub async fn collect_text(mut self) -> ClientResult<String> {
        let mut text = String::new();
        
        while let Some(chunk) = self.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(chunk_data) = chunk.chunk {
                        if let Some(choice) = chunk_data.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                text.push_str(content);
                            }
                        }
                    }
                    
                    if chunk.is_done {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(text)
    }

    /// Get the next chunk
    pub async fn next(&mut self) -> Option<ClientResult<StreamChunk>> {
        self.inner.next().await
    }
}

impl Stream for ChatCompletionStream {
    type Item = ClientResult<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Parse a Server-Sent Events chunk
fn parse_sse_chunk(text: &str) -> ClientResult<StreamChunk> {
    debug!("Parsing SSE chunk: {}", text.trim());
    
    // Handle empty chunks
    if text.trim().is_empty() {
        return Ok(StreamChunk {
            raw: text.to_string(),
            chunk: None,
            is_done: false,
        });
    }
    
    // Look for data: lines in SSE format
    let mut data_content = String::new();
    let mut is_done = false;
    
    for line in text.lines() {
        let line = line.trim();
        
        if line.starts_with("data: ") {
            let data = &line[6..]; // Skip "data: "
            
            // Check for end-of-stream marker
            if data == "[DONE]" {
                is_done = true;
                break;
            }
            
            data_content = data.to_string();
        } else if line.starts_with("event: ") {
            // Handle event types if needed
            debug!("SSE event type: {}", &line[7..]);
        }
    }
    
    // Try to parse as JSON if we have data
    let chunk = if !data_content.is_empty() {
        match serde_json::from_str::<ChatCompletionChunk>(&data_content) {
            Ok(chunk) => {
                debug!("Parsed SSE chunk successfully");
                Some(chunk)
            }
            Err(e) => {
                // If it's not valid JSON, treat as raw data but log warning
                warn!("Failed to parse SSE data as JSON: {} (data: {})", e, data_content);
                None
            }
        }
    } else {
        None
    };
    
    Ok(StreamChunk {
        raw: text.to_string(),
        chunk,
        is_done,
    })
}

/// Helper to parse multiple SSE chunks from a buffer
pub fn parse_sse_buffer(buffer: &str) -> Vec<ClientResult<StreamChunk>> {
    let mut chunks = Vec::new();
    
    // Split by double newlines (SSE chunk separator)
    for chunk_text in buffer.split("\n\n") {
        if !chunk_text.trim().is_empty() {
            chunks.push(parse_sse_chunk(chunk_text));
        }
    }
    
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_chunk() {
        let sse_data = r#"data: {"id":"test","object":"chat.completion.chunk","created":123,"model":"test","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk = parse_sse_chunk(sse_data).unwrap();
        
        assert!(!chunk.is_done);
        assert!(chunk.chunk.is_some());
    }

    #[test]
    fn test_parse_sse_done() {
        let sse_data = "data: [DONE]";
        let chunk = parse_sse_chunk(sse_data).unwrap();
        
        assert!(chunk.is_done);
        assert!(chunk.chunk.is_none());
    }

    #[test]
    fn test_parse_sse_empty() {
        let chunk = parse_sse_chunk("").unwrap();
        
        assert!(!chunk.is_done);
        assert!(chunk.chunk.is_none());
    }

    #[test]
    fn test_parse_sse_invalid_json() {
        let sse_data = "data: invalid json";
        let chunk = parse_sse_chunk(sse_data).unwrap();
        
        assert!(!chunk.is_done);
        assert!(chunk.chunk.is_none());
    }

    #[test]
    fn test_parse_sse_buffer() {
        let buffer = r#"data: {"id":"test","object":"chat.completion.chunk","created":123,"model":"test","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"test","object":"chat.completion.chunk","created":123,"model":"test","choices":[{"index":0,"delta":{"content":" World"},"finish_reason":null}]}

data: [DONE]"#;
        
        let chunks = parse_sse_buffer(buffer);
        assert_eq!(chunks.len(), 3);
        
        // First chunk
        let chunk1 = chunks[0].as_ref().unwrap();
        assert!(!chunk1.is_done);
        assert!(chunk1.chunk.is_some());
        
        // Last chunk
        let chunk3 = chunks[2].as_ref().unwrap();
        assert!(chunk3.is_done);
    }
}