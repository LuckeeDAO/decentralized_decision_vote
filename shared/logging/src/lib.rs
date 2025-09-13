use shared_config::LoggingConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

/// Initialize logging with the given configuration
pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::new(&config.level);
    
    let registry = tracing_subscriber::registry().with(env_filter);
    
    match config.output {
        shared_config::LogOutput::Stdout => {
            let fmt_layer = fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true);
            
            match config.format {
                shared_config::LogFormat::Json => {
                    let json_layer = fmt::layer()
                        .json()
                        .with_target(false)
                        .with_thread_ids(true)
                        .with_thread_names(true);
                    registry.with(json_layer).init();
                }
                shared_config::LogFormat::Text => {
                    registry.with(fmt_layer).init();
                }
            }
        }
        shared_config::LogOutput::File => {
            if let Some(file_path) = &config.file_path {
                match config.format {
                    shared_config::LogFormat::Json => {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(file_path)?;
                        
                        let json_layer = fmt::layer()
                            .json()
                            .with_writer(file)
                            .with_target(false)
                            .with_thread_ids(true)
                            .with_thread_names(true);
                        registry.with(json_layer).init();
                    }
                    shared_config::LogFormat::Text => {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(file_path)?;
                        
                        let fmt_layer = fmt::layer()
                            .with_writer(file)
                            .with_target(false)
                            .with_thread_ids(true)
                            .with_thread_names(true);
                        registry.with(fmt_layer).init();
                    }
                }
            } else {
                // Fallback to stdout if no file path specified
                let fmt_layer = fmt::layer()
                    .with_target(false)
                    .with_thread_ids(true)
                    .with_thread_names(true);
                registry.with(fmt_layer).init();
            }
        }
        shared_config::LogOutput::Both => {
            let stdout_layer = fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true);
            
            let registry = registry.with(stdout_layer);
            
            if let Some(file_path) = &config.file_path {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?;
                
                let file_layer = fmt::layer()
                    .with_writer(file)
                    .with_target(false)
                    .with_thread_ids(true)
                    .with_thread_names(true);
                
                registry.with(file_layer).init();
            } else {
                registry.init();
            }
        }
    }
    
    Ok(())
}

/// Initialize logging with default configuration
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::default();
    init_logging(&config)
}

/// Initialize logging from environment variables
pub fn init_logging_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::from_env();
    init_logging(&config)
}
