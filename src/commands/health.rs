/*!
 * Health Command Implementation
 * 
 * Check server health and status.
 */

use anyhow::Result;
use crate::cli::HealthCommand;
use crate::config::CliConfig;
use crate::output::{OutputFormatter, format_bytes};
use crate::utils::format_duration;
use crate::utils::create_client;

pub async fn handle(cmd: HealthCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    let client = create_client(config, None)?;
    
    output.progress("Checking server health");
    
    let health = client.health().await?;
    
    output.progress_done();
    
    if cmd.detailed {
        // Detailed health information
        output.header("Server Health Status");
        println!();
        
        output.key_value("Status", &health.status);
        
        if let Some(ref version) = health.version {
            output.key_value("Version", version);
        }
        
        if let Some(uptime) = health.uptime_seconds {
            output.key_value("Uptime", &format_duration(uptime));
        }
        
        if let Some(loaded_models) = health.loaded_models {
            output.key_value("Loaded Models", &loaded_models.to_string());
        }
        
        if let Some(ref memory) = health.memory {
            println!();
            output.subheader("Memory Information");
            output.key_value("Total Memory", &format_bytes(memory.total_bytes));
            output.key_value("Used Memory", &format_bytes(memory.used_bytes));
            output.key_value("Available Memory", &format_bytes(memory.available_bytes));
            output.key_value("Model Memory", &format_bytes(memory.model_memory_bytes));
            
            let usage_percent = (memory.used_bytes as f64 / memory.total_bytes as f64) * 100.0;
            output.key_value("Memory Usage", &format!("{:.1}%", usage_percent));
        }
        
        if let Some(ref backends) = health.backends {
            println!();
            output.subheader("Available Backends");
            for (name, info) in backends {
                output.key_value(name, &info.to_string());
            }
        }
    } else {
        // Simple health check
        if health.status == "healthy" {
            output.success("Server is healthy");
        } else {
            output.warning(&format!("Server status: {}", health.status));
        }
        
        if let Some(ref version) = health.version {
            output.info(&format!("Server version: {}", version));
        }
        
        if let Some(loaded_models) = health.loaded_models {
            output.info(&format!("Loaded models: {}", loaded_models));
        }
    }
    
    Ok(())
}