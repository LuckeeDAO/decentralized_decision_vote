use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig { pub host: String, pub port: u16 }

#[derive(Debug, Deserialize, Clone)]
pub struct ApiAuth { pub enabled: bool, pub tokens: Vec<String> }

#[derive(Debug, Deserialize, Clone)]
pub struct Config { pub server: ServerConfig, pub api: ApiAuth }

impl Config {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| format!("read config error: {}", e))?;
        let cfg: Self = serde_yaml::from_str(&content).map_err(|e| format!("parse config error: {}", e))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load_from_env_or_default() -> Result<Self, String> {
        let path = std::env::var("DDV_CONFIG").unwrap_or_else(|_| "./config.yaml".into());
        Self::load_from_path(path)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.host.trim().is_empty() { return Err("server.host cannot be empty".into()); }
        if self.server.port == 0 { return Err("server.port cannot be 0".into()); }
        if self.api.enabled && self.api.tokens.is_empty() { return Err("api.tokens must be non-empty when api.enabled".into()); }
        Ok(())
    }
}
