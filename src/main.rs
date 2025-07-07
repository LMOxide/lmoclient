/*!
 * LMOclient Example CLI
 * 
 * Basic example demonstrating the lmoclient library functionality.
 */

use anyhow::Result;
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

use lmoclient::{LmoClient, ClientConfig};
// Remove unused imports

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("LMOclient Example CLI");

    // Create client
    let config = ClientConfig::default().with_logging(true);
    let client = LmoClient::with_config(config)?;

    // Check server health
    match client.health().await {
        Ok(health) => {
            info!("Server health: {}", health.status);
            if let Some(version) = health.version {
                info!("Server version: {}", version);
            }
        }
        Err(e) => {
            eprintln!("Failed to check server health: {}", e);
            return Ok(());
        }
    }

    // List available models
    match client.list_models().await {
        Ok(models) => {
            info!("Available models: {}", models.models.len());
            for model in &models.models[..std::cmp::min(5, models.models.len())] {
                info!("  - {} ({})", model.id, model.pipeline_tag.as_deref().unwrap_or("unknown"));
            }
        }
        Err(e) => {
            eprintln!("Failed to list models: {}", e);
        }
    }

    // List loaded models
    match client.loaded_models().await {
        Ok(loaded) => {
            info!("Loaded models: {}", loaded.len());
            for model in &loaded {
                info!("  - {} ({}, {}MB)", 
                    model.model_id, 
                    model.status,
                    model.memory_usage_bytes / 1024 / 1024
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to get loaded models: {}", e);
        }
    }

    // Example: Load a small test model (commented out to avoid actual loading)
    /*
    let load_request = LoadModelRequest {
        model: ModelSpecifier::HuggingFace {
            model_id: "microsoft/DialoGPT-small".to_string(),
            revision: None,
        },
        config: None,
        force_reload: Some(false),
    };

    match client.load_model(load_request).await {
        Ok(response) => {
            if response.success {
                info!("Model loaded: {} ({}ms)", response.model_id, response.load_time_ms);
            } else {
                eprintln!("Model loading failed");
            }
        }
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
        }
    }
    */

    // Example: Chat completion (commented out - requires loaded model)
    /*
    let chat_request = client.chat()
        .system("You are a helpful assistant")
        .user("Hello! How are you?")
        .model("microsoft/DialoGPT-small")
        .max_tokens(50)
        .build();

    match client.chat_completion(chat_request).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                info!("Response: {}", choice.message.content);
            }
        }
        Err(e) => {
            eprintln!("Chat completion failed: {}", e);
        }
    }
    */

    info!("Example completed successfully!");
    Ok(())
}
