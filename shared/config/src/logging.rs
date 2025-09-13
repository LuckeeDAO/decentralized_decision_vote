use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File,
    Both,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            output: LogOutput::Stdout,
            file_path: None,
            max_file_size_mb: 100,
            max_files: 5,
        }
    }
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        Self {
            level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            format: match std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string()).as_str() {
                "json" => LogFormat::Json,
                _ => LogFormat::Text,
            },
            output: match std::env::var("LOG_OUTPUT").unwrap_or_else(|_| "stdout".to_string()).as_str() {
                "file" => LogOutput::File,
                "both" => LogOutput::Both,
                _ => LogOutput::Stdout,
            },
            file_path: std::env::var("LOG_FILE_PATH").ok(),
            max_file_size_mb: std::env::var("LOG_MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            max_files: std::env::var("LOG_MAX_FILES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        }
    }
}
