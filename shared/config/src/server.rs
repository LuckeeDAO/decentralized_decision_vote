use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub max_request_size: usize,
    pub request_timeout_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0".to_string(),
            port: 8080,
            cors_origins: vec!["*".to_string()],
            max_request_size: 1024 * 1024, // 1MB
            request_timeout_seconds: 30,
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            bind: std::env::var("SERVER_BIND")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            max_request_size: std::env::var("MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .unwrap_or(1024 * 1024),
            request_timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}
