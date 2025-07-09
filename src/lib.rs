/*!
 * LMOclient Main Library Implementation
 * 
 * HTTP client for communicating with the LMOxide server.
 */

pub mod client;
pub mod config;
pub mod download;
pub mod error;
pub mod models;
pub mod streaming;

// Re-export main types for convenience
pub use client::LmoClient;
pub use config::{ClientConfig, ServerEndpoint};
pub use error::{ClientError, ClientResult};

// Re-export model types
pub use models::*;

// Re-export download types
pub use download::DownloadProgressStream;