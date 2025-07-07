/*!
 * Streaming Chat Completion Support
 * 
 * This module provides streaming support for chat completions.
 */

use crate::error::{ClientError, ClientResult};
use futures::Stream;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamExt;

/// Streaming chat completion response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

/// Stream wrapper for chat completion responses
pub struct ChatCompletionStream {
    response: Response,
}

impl ChatCompletionStream {
    pub fn new(response: Response) -> Self {
        Self { response }
    }

    /// Convert into a stream of chat completion chunks
    pub async fn into_stream(self) -> ClientResult<impl Stream<Item = ClientResult<ChatCompletionChunk>>> {
        let stream = self.response.bytes_stream();
        Ok(stream.map(|result| {
            match result {
                Ok(bytes) => {
                    // Parse SSE format: "data: {json}\n\n"
                    let text = String::from_utf8_lossy(&bytes);
                    
                    // Simple parsing - in production this would be more robust
                    if let Some(json_start) = text.find('{') {
                        if let Some(json_end) = text.rfind('}') {
                            let json_str = &text[json_start..=json_end];
                            serde_json::from_str::<ChatCompletionChunk>(json_str)
                                .map_err(|e| ClientError::ParseError(format!("Failed to parse chunk: {}", e)))
                        } else {
                            Err(ClientError::ParseError("No JSON end found".to_string()))
                        }
                    } else {
                        Err(ClientError::ParseError("No JSON start found".to_string()))
                    }
                }
                Err(e) => Err(ClientError::HttpError(e)),
            }
        }))
    }
}