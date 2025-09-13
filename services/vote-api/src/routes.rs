use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::state::AppState;
use crate::handlers::*;

/// Create the main router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_handler))
        
        // Vote routes
        .route("/api/v1/votes", post(create_vote_handler))
        .route("/api/v1/votes", get(list_votes_handler))
        .route("/api/v1/votes/:id", get(get_vote_handler))
        .route("/api/v1/votes/:id/results", get(get_results_handler))
        .route("/api/v1/votes/:id/verify", get(verify_results_handler))
        
        // Commitment routes
        .route("/api/v1/votes/:id/commit", post(commit_vote_handler))
        
        // Reveal routes
        .route("/api/v1/votes/:id/reveal", post(reveal_vote_handler))
        
        // Template routes
        .route("/api/v1/templates", get(list_templates_handler))
        .route("/api/v1/templates/:id", get(get_template_handler))
        
        // WebSocket routes
        .route("/ws/votes/:id", get(websocket_handler))
        
        .with_state(state)
}
