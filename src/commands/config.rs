/*!
 * Config Command Implementation
 * 
 * Manage CLI configuration.
 */

use anyhow::Result;
use crate::cli::{ConfigCommand, ConfigAction};
use crate::config::CliConfig;
use crate::output::OutputFormatter;

pub async fn handle(cmd: ConfigCommand, config: &CliConfig) -> Result<()> {
    let output = OutputFormatter::new(config, None, false);
    
    match cmd.action {
        ConfigAction::Show => {
            output.header("Current Configuration");
            println!();
            output.print(config)?;
        }
        ConfigAction::Set { key, value } => {
            let mut new_config = config.clone();
            new_config.set_value(&key, &value)?;
            new_config.save()?;
            output.success(&format!("Set {} = {}", key, value));
        }
        ConfigAction::Get { key } => {
            let value = config.get_value(&key)?;
            println!("{}", value);
        }
        ConfigAction::Init => {
            let default_config = CliConfig::default();
            default_config.save()?;
            output.success("Configuration initialized with defaults");
        }
        ConfigAction::Reset => {
            let default_config = CliConfig::default();
            default_config.save()?;
            output.success("Configuration reset to defaults");
        }
    }
    
    Ok(())
}