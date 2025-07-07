/*!
 * Chat Command Implementation
 * 
 * Interactive chat with loaded models.
 */

use anyhow::Result;
use crate::cli::ChatCommand;
use crate::config::CliConfig;
use crate::output::OutputFormatter;

pub async fn handle(_cmd: ChatCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    output.warning("Chat command not yet implemented");
    Ok(())
}