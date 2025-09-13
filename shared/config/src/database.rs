use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:./data/votes.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 600,
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/votes.db".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            connection_timeout_seconds: std::env::var("DB_CONNECTION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            idle_timeout_seconds: std::env::var("DB_IDLE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
        }
    }
}
