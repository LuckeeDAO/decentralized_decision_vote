use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::{info, error, debug};

use shared_types::*;
use crate::state::AppState;

/// Health check handler
pub async fn health_handler() -> Result<Json<HealthResponse>, StatusCode> {
    let mut services = std::collections::HashMap::new();
    services.insert("vote-api".to_string(), ServiceStatus {
        status: "healthy".to_string(),
        message: None,
    });
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    };
    
    Ok(Json(response))
}

/// Create a new vote
pub async fn create_vote_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateVoteRequest>,
) -> Result<Json<CreateVoteResponse>, StatusCode> {
    info!("Creating new vote: {}", request.config.title);
    
    match state.vote_engine.create_vote(request.config).await {
        Ok(vote_id) => {
            let response = CreateVoteResponse {
                vote_id,
                success: true,
                message: "Vote created successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to create vote: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a vote by ID
pub async fn get_vote_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GetVoteResponse>, StatusCode> {
    debug!("Getting vote: {}", id);
    
    match state.vote_engine.get_vote(&id).await {
        Ok(vote) => {
            let response = GetVoteResponse {
                vote,
                success: true,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get vote {}: {}", id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// List votes with pagination
pub async fn list_votes_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListVotesResponse>, StatusCode> {
    debug!("Listing votes: page={}, size={}", query.page, query.page_size);
    
    match state.vote_engine.list_votes(query).await {
        Ok(votes) => {
            let response = ListVotesResponse {
                votes,
                success: true,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list votes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get vote results
pub async fn get_results_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GetResultsResponse>, StatusCode> {
    debug!("Getting results for vote: {}", id);
    
    match state.vote_engine.get_results(&id).await {
        Ok(results) => {
            let response = GetResultsResponse {
                results,
                success: true,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get results for vote {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Submit a commitment
pub async fn commit_vote_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<CommitRequest>,
) -> Result<Json<CommitResponse>, StatusCode> {
    info!("Processing commitment for vote: {}", id);
    
    match state.vote_engine.commit_vote(&id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process commitment for vote {}: {}", id, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Submit a reveal
pub async fn reveal_vote_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<RevealRequest>,
) -> Result<Json<RevealResponse>, StatusCode> {
    info!("Processing reveal for vote: {}", id);
    
    match state.vote_engine.reveal_vote(&id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process reveal for vote {}: {}", id, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// List available templates
pub async fn list_templates_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Listing templates");
    
    let templates = state.template_registry.list();
    let response = serde_json::json!({
        "templates": templates,
        "success": true
    });
    
    Ok(Json(response))
}

/// Verify vote results
pub async fn verify_results_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<VerifyResultsResponse>, StatusCode> {
    debug!("Verifying results for vote: {}", id);
    
    match state.vote_engine.verify_results(&id).await {
        Ok(verification) => {
            let response = VerifyResultsResponse {
                verification,
                success: true,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to verify results for vote {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get template details
pub async fn get_template_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting template: {}", id);
    
    match state.template_registry.get(&id) {
        Ok(template) => {
            let response = serde_json::json!({
                "id": template.id(),
                "name": template.name(),
                "description": template.description(),
                "schema": template.get_schema(),
                "success": true
            });
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get template {}: {}", id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// WebSocket handler for real-time updates
pub async fn websocket_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, StatusCode> {
    debug!("WebSocket connection for vote: {}", id);
    
    // TODO: Implement WebSocket handler for real-time vote updates
    // For now, just return a simple response
    Ok(ws.on_upgrade(|socket| async move {
        // Handle WebSocket connection
        info!("WebSocket connection established for vote: {}", id);
        
        // Close the connection immediately for now
        drop(socket);
    }))
}
