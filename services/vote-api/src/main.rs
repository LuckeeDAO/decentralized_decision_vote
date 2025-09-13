use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::info;

use shared_logging::init_logging_from_env;
use shared_config::AppConfig;
use crate::routes::create_router;
use crate::state::AppState;

mod routes;
mod handlers;
mod middleware;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_from_env()?;
    
    info!("Starting Vote API service");
    
    // Load configuration
    let config = AppConfig::load_from_env()?;
    
    // Initialize application state
    let state = AppState::new(config).await?;
    
    // Extract server configuration before moving state
    let server_config = state.config.server.clone();
    
    // Create router
    let app: Router = create_router(Arc::new(state))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());
    
    // Start server
    let addr: SocketAddr = format!("{}:{}", server_config.bind, server_config.port)
        .parse()
        .expect("Invalid server address");
    
    info!("Vote API listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("signal received, starting graceful shutdown");
}
