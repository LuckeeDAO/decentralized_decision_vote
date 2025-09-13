use notification_service::{NotificationService, NotificationConfig};
use shared_logging::init_logging_from_env;
use tracing::{info, error};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging_from_env().map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
    
    info!("Starting Notification Service");
    
    // Load configuration
    let config = NotificationConfig::default();
    info!("Configuration loaded successfully");
    
    // Create and start notification service
    let mut service = NotificationService::new(config).await?;
    
    info!("Notification service is running...");
    
    // Start the service
    if let Err(e) = service.start().await {
        error!("Failed to start notification service: {}", e);
        return Err(e.into());
    }
    
    // Keep the service running
    tokio::signal::ctrl_c().await?;
    info!("Notification service shutting down");
    
    service.shutdown().await?;
    info!("Notification service stopped");
    
    Ok(())
}
