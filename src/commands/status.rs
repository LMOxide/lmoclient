/*!
 * Status Command Implementation
 * 
 * Show status of loaded models.
 */

use anyhow::Result;
use crate::cli::StatusCommand;
use crate::config::CliConfig;
use crate::output::OutputFormatter;

pub async fn handle(_cmd: StatusCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    output.warning("Status command not yet implemented");
    Ok(())
}