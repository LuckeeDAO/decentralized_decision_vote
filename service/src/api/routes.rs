/**
 * 投票服务API路由模块
 * 
 * 这个模块定义了Luckee DAO投票系统的所有HTTP API端点，包括：
 * 1. 系统状态查询
 * 2. 投票会话管理
 * 3. 投票提交和揭示
 * 4. 结果查询和统计
 * 5. WebSocket实时通信
 * 
 * 技术特性：
 * - 使用Axum框架构建高性能异步API
 * - 支持WebSocket实时数据推送
 * - 完整的错误处理和响应格式化
 * - 支持分页查询和过滤
 * - 集成区块链状态同步
 */

use axum::{routing::get, Router, Json};
use axum::extract::ws::{WebSocketUpgrade, Message, WebSocket};
use std::sync::Arc;
use crate::core::state::AppState;
use axum::extract::{State, Path, Query};
use crate::model::response::ApiResponse;
use crate::model::vote::*;

/**
 * 系统状态查询处理器
 * 返回当前系统的运行状态和配置信息
 * 
 * @param state - 应用状态，包含系统配置和区块链连接
 * @returns JSON响应，包含系统状态信息
 */
async fn status_handler(State(state): State<Arc<AppState>>) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(Some(state.get_status_json().await)))
}

/**
 * 区块链高度查询处理器
 * 返回当前区块链的最新高度信息
 * 
 * @param state - 应用状态，包含当前区块高度
 * @returns JSON响应，包含当前区块高度
 */
async fn height_handler(State(state): State<Arc<AppState>>) -> Json<ApiResponse<ChainHeightDto>> {
    let h = state.current_height.load(std::sync::atomic::Ordering::Relaxed);
    Json(ApiResponse::success(Some(ChainHeightDto { height: h })))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/status", get(status_handler))
        .route("/api/height", get(height_handler))
        .route("/api/ws/height", get(ws_height))
        .route("/api/votes", get(list_votes).post(create_vote))
        .route("/api/votes/:id", get(get_vote))
        .route("/api/votes/:id/commit", axum::routing::post(commit_vote))
        .route("/api/votes/:id/reveal", axum::routing::post(reveal_vote))
        .route("/api/votes/:id/results", get(results_vote))
        .with_state(state)
}

async fn list_votes(State(state): State<Arc<AppState>>, Query(q): Query<PaginationQuery>) -> Json<ApiResponse<Page<VoteSummaryDto>>> {
    let offset = q.offset.unwrap_or(0);
    let limit = q.limit.unwrap_or(50);
    match state.service.list_votes(offset, limit).await {
        Ok((items, total)) => Json(ApiResponse::success(Some(Page { items, total }))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn create_vote(State(state): State<Arc<AppState>>, Json(req): Json<CreateVoteRequest>) -> Json<ApiResponse<String>> {
    // simple input guard
    if req.config.title.trim().is_empty() {
        return Json(ApiResponse::error("title cannot be empty"));
    }
    if req.config.commit_end_height > req.config.reveal_start_height {
        return Json(ApiResponse::error("commit window must end before reveal starts"));
    }
    match state.service.create_vote(req.config).await {
        Ok(id) => Json(ApiResponse::success(Some(id))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn get_vote(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<ApiResponse<VoteDetailDto>> {
    match state.service.get_vote(&id).await {
        Ok(v) => Json(ApiResponse::success(Some(v))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn commit_vote(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<CommitRequest>) -> Json<ApiResponse<CommitResponse>> {
    if req.voter.trim().is_empty() { return Json(ApiResponse::error("voter is required")); }
    if req.salt_hex.len() < 2 { return Json(ApiResponse::error("salt_hex is required")); }
    match state.service.commit(&id, &req.voter, req.vote_value, req.salt_hex).await {
        Ok(r) => Json(ApiResponse::success(Some(r))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn reveal_vote(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<RevealRequest>) -> Json<ApiResponse<RevealResponse>> {
    if req.voter.trim().is_empty() { return Json(ApiResponse::error("voter is required")); }
    if req.salt_hex.len() < 2 { return Json(ApiResponse::error("salt_hex is required")); }
    match state.service.reveal(&id, &req.voter, req.vote_value, req.salt_hex).await {
        Ok(r) => Json(ApiResponse::success(Some(r))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn results_vote(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<ApiResponse<VoteResultsDto>> {
    match state.service.results(&id).await {
        Ok(r) => Json(ApiResponse::success(Some(r))),
        Err(e) => Json(ApiResponse::error(&format!("{}", e))),
    }
}

async fn ws_height(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| ws_height_loop(state, socket))
}

async fn ws_height_loop(state: Arc<AppState>, mut socket: WebSocket) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let h = state.current_height.load(std::sync::atomic::Ordering::Relaxed);
        let payload = serde_json::json!({"type": "height", "height": h});
        if socket.send(Message::Text(payload.to_string())).await.is_err() {
            break;
        }
    }
}
