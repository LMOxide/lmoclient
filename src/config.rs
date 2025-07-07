/*!
 * CLI Configuration Management
 *
 * Handles loading, saving, and managing CLI configuration.
 */

use std::path::PathBuf;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use dirs::config_dir;

use crate::error::CliError;

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default server URL
    pub server_url: String,

    /// Default output format
    pub output_format: String,

    /// Enable colors by default
    pub enable_colors: bool,

    /// Default chat settings
    pub chat: ChatConfig,

    /// Default model settings
    pub models: ModelsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Default temperature
    pub temperature: f32,

    /// Default max tokens
    pub max_tokens: u32,

    /// Enable streaming by default
    pub stream: bool,

    /// Default system prompt
    pub system_prompt: Option<String>,

    /// Auto-save conversations
    pub auto_save: bool,

    /// Conversation history directory
    pub history_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Default search limit
    pub default_limit: u32,

    /// Default sort field
    pub default_sort: String,

    /// Default sort direction
    pub default_direction: String,

    /// Preferred model providers
    pub preferred_providers: Vec<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            output_format: "table".to_string(),
            enable_colors: true,
            chat: ChatConfig {
                temperature: 0.7,
                max_tokens: 1000,
                stream: true,
                system_prompt: None,
                auto_save: false,
                history_dir: None,
            },
            models: ModelsConfig {
                default_limit: 20,
                default_sort: "downloads".to_string(),
                default_direction: "desc".to_string(),
                preferred_providers: vec![
                    "microsoft".to_string(),
                    "meta-llama".to_string(),
                    "huggingface".to_string(),
                ],
            },
        }
    }
}

impl CliConfig {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

            toml::from_str(&content)
                .with_context(|| "Failed to parse config file")
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get the configuration file path
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| CliError::ConfigError("Could not find config directory".to_string()))?;

        Ok(config_dir.join("lmo").join("config.toml"))
    }

    /// Get server URL with fallback
    pub fn server_url<'a>(&'a self, override_url: Option<&'a str>) -> &'a str {
        override_url.unwrap_or(&self.server_url)
    }

    /// Set a configuration value by key
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "server_url" => self.server_url = value.to_string(),
            "output_format" => self.output_format = value.to_string(),
            "enable_colors" => self.enable_colors = value.parse()
                .with_context(|| "Invalid boolean value for enable_colors")?,
            "chat.temperature" => self.chat.temperature = value.parse()
                .with_context(|| "Invalid float value for chat.temperature")?,
            "chat.max_tokens" => self.chat.max_tokens = value.parse()
                .with_context(|| "Invalid integer value for chat.max_tokens")?,
            "chat.stream" => self.chat.stream = value.parse()
                .with_context(|| "Invalid boolean value for chat.stream")?,
            "chat.system_prompt" => self.chat.system_prompt = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            },
            "chat.auto_save" => self.chat.auto_save = value.parse()
                .with_context(|| "Invalid boolean value for chat.auto_save")?,
            "chat.history_dir" => self.chat.history_dir = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            },
            "models.default_limit" => self.models.default_limit = value.parse()
                .with_context(|| "Invalid integer value for models.default_limit")?,
            "models.default_sort" => self.models.default_sort = value.to_string(),
            "models.default_direction" => self.models.default_direction = value.to_string(),
            _ => return Err(CliError::ConfigError(format!("Unknown config key: {}", key)).into()),
        }

        // Validate retries
        if self.max_retries > 10 {
            return Err(ClientError::ConfigError("Max retries should not exceed 10".to_string()));
        }

        Ok(())
    }
}

/// Server endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Description
    pub description: Option<String>,
}

    /// Get a configuration value by key
    pub fn get_value(&self, key: &str) -> Result<String> {
        let value = match key {
            "server_url" => self.server_url.clone(),
            "output_format" => self.output_format.clone(),
            "enable_colors" => self.enable_colors.to_string(),
            "chat.temperature" => self.chat.temperature.to_string(),
            "chat.max_tokens" => self.chat.max_tokens.to_string(),
            "chat.stream" => self.chat.stream.to_string(),
            "chat.system_prompt" => self.chat.system_prompt.as_deref().unwrap_or("").to_string(),
            "chat.auto_save" => self.chat.auto_save.to_string(),
            "chat.history_dir" => self.chat.history_dir.as_deref().unwrap_or("").to_string(),
            "models.default_limit" => self.models.default_limit.to_string(),
            "models.default_sort" => self.models.default_sort.clone(),
            "models.default_direction" => self.models.default_direction.clone(),
            _ => return Err(CliError::ConfigError(format!("Unknown config key: {}", key)).into()),
        };
        Ok(value)
    }

    /// List all configuration keys
    pub fn list_keys() -> Vec<&'static str> {
        vec![
            "server_url",
            "output_format",
            "enable_colors",
            "chat.temperature",
            "chat.max_tokens",
            "chat.stream",
            "chat.system_prompt",
            "chat.auto_save",
            "chat.history_dir",
            "models.default_limit",
            "models.default_sort",
            "models.default_direction",
        ]
    }
}