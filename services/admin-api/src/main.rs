use admin_api::{AdminApiService, AdminConfig, AdminError};
use shared_logging::init_logging_from_env;
use shared_config::AppConfig;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), AdminError> {
    // Initialize logging
    init_logging_from_env()?;
    
    info!("Starting Admin API Service");
    
    // Load configuration
    let _config = AppConfig::load_from_env()?;
    info!("Configuration loaded successfully");
    
    // Create and start admin API service
    let admin_config = AdminConfig::default(); // TODO: Convert from AppConfig
    let mut service = AdminApiService::new(admin_config).await?;
    
    info!("Admin API service is running...");
    
    // Start the service
    if let Err(e) = service.start().await {
        error!("Failed to start admin API service: {}", e);
        return Err(e);
    }
    
    // Keep the service running
    tokio::signal::ctrl_c().await?;
    info!("Admin API service shutting down");
    
    service.shutdown().await?;
    info!("Admin API service stopped");
    
    Ok(())
}
