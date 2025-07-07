/*!
 * CLI Utilities
 * 
 * Helper functions and utilities for CLI operations.
 */

use std::io::{self, Write};
use anyhow::Result;
use dialoguer::{Select, Confirm, Input};
use lmoclient::{LmoClient, ClientConfig, ModelInfo};

use crate::config::CliConfig;
use crate::error::CliError;
use crate::output::OutputFormatter;

/// Create an HTTP client from CLI configuration
pub fn create_client(config: &CliConfig, server_url_override: Option<&str>) -> Result<LmoClient> {
    let server_url = config.server_url(server_url_override);
    
    let client_config = ClientConfig::new(server_url)?
        .with_logging(true);
    
    Ok(LmoClient::with_config(client_config)?)
}

/// Interactive model selection
pub async fn select_model(client: &LmoClient, output: &OutputFormatter) -> Result<String> {
    output.progress("Fetching available models");
    
    let models_response = client.list_models().await
        .map_err(|e| CliError::ServerError(format!("Failed to fetch models: {}", e)))?;
    
    output.progress_done();
    
    if models_response.models.is_empty() {
        return Err(CliError::ModelNotFound("No models available".to_string()).into());
    }
    
    let model_choices: Vec<String> = models_response.models
        .iter()
        .map(|m| format!("{} ({})", m.id, m.pipeline_tag.as_deref().unwrap_or("unknown")))
        .collect();
    
    let selection = Select::new()
        .with_prompt("Select a model")
        .items(&model_choices)
        .default(0)
        .interact()?;
    
    Ok(models_response.models[selection].id.clone())
}

/// Interactive model selection for loaded models
pub async fn select_loaded_model(client: &LmoClient, output: &OutputFormatter) -> Result<String> {
    output.progress("Fetching loaded models");
    
    let loaded_models = client.loaded_models().await
        .map_err(|e| CliError::ServerError(format!("Failed to fetch loaded models: {}", e)))?;
    
    output.progress_done();
    
    if loaded_models.is_empty() {
        return Err(CliError::ModelNotFound("No models are currently loaded".to_string()).into());
    }
    
    let model_choices: Vec<String> = loaded_models
        .iter()
        .map(|m| format!("{} ({})", m.model_id, m.status))
        .collect();
    
    let selection = Select::new()
        .with_prompt("Select a loaded model")
        .items(&model_choices)
        .default(0)
        .interact()?;
    
    Ok(loaded_models[selection].model_id.clone())
}

/// Confirm action with user
pub fn confirm_action(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(message)
        .default(default)
        .interact()?)
}

/// Get user input
pub fn get_input(prompt: &str, default: Option<&str>) -> Result<String> {
    let mut input_builder = Input::<String>::new().with_prompt(prompt);
    
    if let Some(default_value) = default {
        input_builder = input_builder.default(default_value.to_string());
    }
    
    Ok(input_builder.interact_text()?)
}

/// Validate server URL format
pub fn validate_server_url(url: &str) -> Result<()> {
    url::Url::parse(url)
        .map_err(|e| CliError::InvalidInput(format!("Invalid server URL: {}", e)))?;
    Ok(())
}

/// Format model information for display
pub fn format_model_info(model: &ModelInfo) -> String {
    let mut lines = Vec::new();
    
    lines.push(format!("ID: {}", model.id));
    
    if let Some(ref author) = model.author {
        lines.push(format!("Author: {}", author));
    }
    
    lines.push(format!("Downloads: {}", crate::output::format_number(model.downloads)));
    
    if let Some(ref pipeline) = model.pipeline_tag {
        lines.push(format!("Pipeline: {}", pipeline));
    }
    
    if let Some(ref library) = model.library_name {
        lines.push(format!("Library: {}", library));
    }
    
    if !model.tags.is_empty() {
        lines.push(format!("Tags: {}", model.tags.join(", ")));
    }
    
    lines.push(format!("Created: {}", model.created_at));
    lines.push(format!("Updated: {}", model.updated_at));
    
    lines.join("\n")
}

/// Check if the server is accessible
pub async fn check_server_health(client: &LmoClient, output: &OutputFormatter) -> Result<()> {
    output.progress("Checking server health");
    
    match client.health().await {
        Ok(health) => {
            output.progress_done();
            output.success(&format!("Server is healthy ({})", health.status));
            Ok(())
        }
        Err(e) => {
            output.progress_failed(&e.to_string());
            Err(CliError::ServerError(format!("Server health check failed: {}", e)).into())
        }
    }
}

/// Wait for user input to continue
pub fn wait_for_enter(message: &str) {
    print!("{} Press Enter to continue...", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// Format duration in human-readable form
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}