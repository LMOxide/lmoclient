/*!
 * Unload Command Implementation
 * 
 * Unload loaded models.
 */

use anyhow::Result;
use crate::cli::UnloadCommand;
use crate::config::CliConfig;
use crate::output::OutputFormatter;

pub async fn handle(_cmd: UnloadCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    output.warning("Unload command not yet implemented");
    Ok(())
}