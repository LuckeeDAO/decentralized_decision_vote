pub mod server;
pub mod database;
pub mod blockchain;
pub mod logging;

pub use server::*;
pub use database::*;
pub use blockchain::*;
pub use logging::*;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub blockchain: Option<BlockchainConfig>,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_from_env() -> anyhow::Result<Self> {
        let config = AppConfig {
            server: ServerConfig::from_env(),
            database: DatabaseConfig::from_env(),
            blockchain: BlockchainConfig::from_env().ok(),
            logging: LoggingConfig::from_env(),
        };
        Ok(config)
    }
}
