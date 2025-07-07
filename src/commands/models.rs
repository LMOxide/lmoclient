/*!
 * Models Command Implementation
 * 
 * List and search available models.
 */

use anyhow::Result;
use crate::cli::ModelsCommand;
use crate::config::CliConfig;
use crate::output::{OutputFormatter, format_number, truncate_text};
use crate::utils::{create_client, check_server_health};

pub async fn handle(cmd: ModelsCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    let client = create_client(config, None)?;
    
    // Check server health first
    check_server_health(&client, &output).await?;
    
    output.progress("Fetching models");
    
    // Fetch models with filters
    let models_response = client.list_models().await?;
    
    output.progress_done();
    
    if models_response.models.is_empty() {
        output.warning("No models found matching the criteria");
        return Ok(());
    }
    
    // Apply client-side filtering and sorting
    let mut models = models_response.models;
    
    // Filter by search term
    if let Some(ref search) = cmd.search {
        models.retain(|m| m.id.to_lowercase().contains(&search.to_lowercase()));
    }
    
    // Filter by author
    if let Some(ref author) = cmd.author {
        models.retain(|m| {
            m.author.as_ref()
                .map(|a| a.to_lowercase().contains(&author.to_lowercase()))
                .unwrap_or(false)
        });
    }
    
    // Filter by tags
    if let Some(ref tags) = cmd.tags {
        let search_tags: Vec<&str> = tags.split(',').map(|t| t.trim()).collect();
        models.retain(|m| {
            search_tags.iter().any(|tag| {
                m.tags.iter().any(|t| t.to_lowercase().contains(&tag.to_lowercase()))
            })
        });
    }
    
    // Filter by pipeline
    if let Some(ref pipeline) = cmd.pipeline {
        models.retain(|m| {
            m.pipeline_tag.as_ref()
                .map(|p| p.to_lowercase().contains(&pipeline.to_lowercase()))
                .unwrap_or(false)
        });
    }
    
    // Sort models
    match cmd.sort.as_str() {
        "downloads" => {
            if cmd.direction == "asc" {
                models.sort_by(|a, b| a.downloads.cmp(&b.downloads));
            } else {
                models.sort_by(|a, b| b.downloads.cmp(&a.downloads));
            }
        }
        "author" => {
            if cmd.direction == "asc" {
                models.sort_by(|a, b| a.author.cmp(&b.author));
            } else {
                models.sort_by(|a, b| b.author.cmp(&a.author));
            }
        }
        "created" => {
            if cmd.direction == "asc" {
                models.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            } else {
                models.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }
        _ => {} // Keep original order
    }
    
    // Limit results
    models.truncate(cmd.limit as usize);
    
    // Display results
    output.header(&format!("Available Models ({} found)", models.len()));
    println!();
    
    match &config.output_format[..] {
        "json" => {
            output.print(&models)?;
        }
        "yaml" => {
            output.print(&models)?;
        }
        _ => {
            // Table format
            println!("{:<40} {:<20} {:<15} {:<20} {:<30}", 
                "Model ID", "Author", "Downloads", "Pipeline", "Tags");
            println!("{}", "-".repeat(125));
            
            for model in &models {
                let author = model.author.as_deref().unwrap_or("Unknown");
                let pipeline = model.pipeline_tag.as_deref().unwrap_or("Unknown");
                let tags = if model.tags.is_empty() {
                    "None".to_string()
                } else {
                    truncate_text(&model.tags.join(", "), 30)
                };
                
                println!("{:<40} {:<20} {:<15} {:<20} {:<30}", 
                    truncate_text(&model.id, 40),
                    truncate_text(author, 20),
                    format_number(model.downloads),
                    truncate_text(pipeline, 20),
                    tags
                );
            }
        }
    }
    
    println!();
    output.info(&format!("Showing {} of {} total models", models.len(), models_response.total.unwrap_or(models.len() as u32)));
    
    Ok(())
}