use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub api_url: String,
    pub default_template: String,
    pub default_commitment_hours: u32,
    pub default_reveal_hours: u32,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Table,
    Pretty,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8080".to_string(),
            default_template: "yes_no".to_string(),
            default_commitment_hours: 24,
            default_reveal_hours: 24,
            output_format: OutputFormat::Pretty,
        }
    }
}

impl CliConfig {
    /// Load configuration from file
    #[allow(dead_code)]
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: CliConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Get default config file path
    #[allow(dead_code)]
    pub fn default_config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".vote-cli");
        path.push("config.json");
        path
    }
}
