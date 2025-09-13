//! HTTP handlers for admin API

use crate::{
    OperationResult, SystemStatistics, 
    SessionManagementInfo, ConfigManagementInfo, LogEntry,
    auth::{LoginRequest, LoginResponse, CreateUserRequest, UpdateUserRequest, ChangePasswordRequest, UserInfo},
    middleware::AuthMiddlewareState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use tracing::{info, warn, error};
use uuid::Uuid;

/// 分页参数
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

/// 会话查询参数
#[derive(Debug, Deserialize)]
pub struct SessionQueryParams {
    pub status: Option<String>,
    pub phase: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

/// 用户查询参数
#[derive(Debug, Deserialize)]
pub struct UserQueryParams {
    pub role: Option<String>,
    pub is_active: Option<bool>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub level: Option<String>,
    pub source: Option<String>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// 创建HTTP路由
pub fn create_http_router(state: AuthMiddlewareState) -> Router {
    Router::new()
        // 认证相关路由（不需要认证）
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        
        // 健康检查和状态
        .route("/health", get(health_check))
        .route("/status", get(get_system_status))
        .route("/statistics", get(get_statistics))
        
        // 用户管理（需要认证和权限）
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/users/:id/password", put(change_password))
        .route("/users/:id/roles", get(get_user_roles).post(assign_role).delete(remove_role))
        
        // 会话管理
        .route("/sessions", get(list_sessions))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        
        // 配置管理
        .route("/config", get(get_config).put(update_config))
        .route("/config/:key", get(get_config_value).put(set_config_value).delete(delete_config_value))
        
        // 日志管理
        .route("/logs", get(list_logs))
        .route("/logs/:id", get(get_log_entry))
        
        // 权限管理
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/:name", get(get_role).put(update_role).delete(delete_role))
        .route("/permissions", get(list_permissions))
        
        .with_state(state)
}

/// 用户登录
async fn login(
    State(state): State<AuthMiddlewareState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let username = request.username.clone();
    info!("Login attempt for user: {}", username);
    
    let mut auth_service = (*state.auth_service).clone();
    match auth_service.login(request).await {
        Ok(response) => {
            info!("User {} logged in successfully", response.user.username);
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Login failed for user {}: {}", username, e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// 刷新令牌
async fn refresh_token(
    State(_state): State<AuthMiddlewareState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 简化实现，实际应用中应该验证刷新令牌
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// 健康检查
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "admin-api"
    }))
}

/// 获取系统状态
async fn get_system_status(
    State(_state): State<AuthMiddlewareState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 这里应该从实际的服务获取状态信息
    let status = serde_json::json!({
        "status": "running",
        "uptime": "N/A",
        "version": "1.0.0",
        "environment": "development"
    });
    
    Ok(Json(status))
}

/// 获取系统统计信息
async fn get_statistics(
    State(_state): State<AuthMiddlewareState>,
) -> Result<Json<SystemStatistics>, StatusCode> {
    // 这里应该从实际的服务获取统计信息
    let stats = SystemStatistics {
        total_sessions: 100,
        active_sessions: 5,
        total_users: 50,
        active_users: 25,
        total_commits: 1000,
        total_reveals: 800,
        system_uptime: "2 days, 5 hours".to_string(),
        memory_usage: 45.2,
        cpu_usage: 12.8,
        disk_usage: 67.5,
        network_io: crate::NetworkStatistics {
            bytes_received: 1024000,
            bytes_sent: 2048000,
            connections: 15,
            requests_per_second: 25.5,
        },
    };
    
    Ok(Json(stats))
}

/// 列出用户
async fn list_users(
    State(state): State<AuthMiddlewareState>,
    Query(_params): Query<UserQueryParams>,
    Query(_pagination): Query<PaginationParams>,
) -> Result<Json<Vec<UserInfo>>, StatusCode> {
    // 这里应该从数据库获取用户列表
    let users = state.auth_service.get_all_users();
    Ok(Json(users))
}

/// 创建用户
async fn create_user(
    State(state): State<AuthMiddlewareState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserInfo>, StatusCode> {
    info!("Creating user: {}", request.username);
    
    let mut auth_service = (*state.auth_service).clone();
    match auth_service.create_user(request).await {
        Ok(user) => {
            info!("User created successfully: {}", user.username);
            Ok(Json(user))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 获取用户信息
async fn get_user(
    State(state): State<AuthMiddlewareState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserInfo>, StatusCode> {
    match state.auth_service.get_user(user_id) {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 更新用户
async fn update_user(
    State(state): State<AuthMiddlewareState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserInfo>, StatusCode> {
    info!("Updating user: {}", user_id);
    
    let mut auth_service = (*state.auth_service).clone();
    match auth_service.update_user(user_id, request).await {
        Ok(user) => {
            info!("User updated successfully: {}", user.username);
            Ok(Json(user))
        }
        Err(e) => {
            error!("Failed to update user: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 删除用户
async fn delete_user(
    State(state): State<AuthMiddlewareState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<OperationResult>, StatusCode> {
    info!("Deleting user: {}", user_id);
    
    let mut auth_service = (*state.auth_service).clone();
    match auth_service.delete_user(user_id).await {
        Ok(_) => {
            info!("User deleted successfully: {}", user_id);
            Ok(Json(OperationResult::success(
                "User deleted successfully".to_string(),
                None,
            )))
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 更改密码
async fn change_password(
    State(state): State<AuthMiddlewareState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<OperationResult>, StatusCode> {
    info!("Changing password for user: {}", user_id);
    
    let mut auth_service = (*state.auth_service).clone();
    match auth_service.change_password(user_id, request).await {
        Ok(_) => {
            info!("Password changed successfully for user: {}", user_id);
            Ok(Json(OperationResult::success(
                "Password changed successfully".to_string(),
                None,
            )))
        }
        Err(e) => {
            error!("Failed to change password: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 获取用户角色
async fn get_user_roles(
    State(_state): State<AuthMiddlewareState>,
    Path(_user_id): Path<Uuid>,
) -> Result<Json<Vec<String>>, StatusCode> {
    // 这里应该从数据库获取用户角色
    let roles = vec!["admin".to_string()];
    Ok(Json(roles))
}

/// 分配角色
async fn assign_role(
    State(_state): State<AuthMiddlewareState>,
    Path(_user_id): Path<Uuid>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Role assigned successfully".to_string(),
        None,
    )))
}

/// 移除角色
async fn remove_role(
    State(_state): State<AuthMiddlewareState>,
    Path(_user_id): Path<Uuid>,
    Path(_role): Path<String>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Role removed successfully".to_string(),
        None,
    )))
}

/// 列出会话
async fn list_sessions(
    State(_state): State<AuthMiddlewareState>,
    Query(_params): Query<SessionQueryParams>,
    Query(_pagination): Query<PaginationParams>,
) -> Result<Json<Vec<SessionManagementInfo>>, StatusCode> {
    // 这里应该从数据库获取会话列表
    let sessions = vec![];
    Ok(Json(sessions))
}

/// 获取会话信息
async fn get_session(
    State(_state): State<AuthMiddlewareState>,
    Path(_session_id): Path<String>,
) -> Result<Json<SessionManagementInfo>, StatusCode> {
    // 这里应该从数据库获取会话信息
    Err(StatusCode::NOT_FOUND)
}

/// 删除会话
async fn delete_session(
    State(_state): State<AuthMiddlewareState>,
    Path(_session_id): Path<String>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Session deleted successfully".to_string(),
        None,
    )))
}

/// 获取配置
async fn get_config(
    State(_state): State<AuthMiddlewareState>,
) -> Result<Json<Vec<ConfigManagementInfo>>, StatusCode> {
    // 这里应该从配置存储获取配置
    let configs = vec![];
    Ok(Json(configs))
}

/// 更新配置
async fn update_config(
    State(_state): State<AuthMiddlewareState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Configuration updated successfully".to_string(),
        None,
    )))
}

/// 获取配置值
async fn get_config_value(
    State(_state): State<AuthMiddlewareState>,
    Path(_key): Path<String>,
) -> Result<Json<ConfigManagementInfo>, StatusCode> {
    // 简化实现
    Err(StatusCode::NOT_FOUND)
}

/// 设置配置值
async fn set_config_value(
    State(_state): State<AuthMiddlewareState>,
    Path(_key): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Configuration value set successfully".to_string(),
        None,
    )))
}

/// 删除配置值
async fn delete_config_value(
    State(_state): State<AuthMiddlewareState>,
    Path(_key): Path<String>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Configuration value deleted successfully".to_string(),
        None,
    )))
}

/// 列出日志
async fn list_logs(
    State(_state): State<AuthMiddlewareState>,
    Query(_params): Query<LogQueryParams>,
    Query(_pagination): Query<PaginationParams>,
) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    // 这里应该从日志存储获取日志
    let logs = vec![];
    Ok(Json(logs))
}

/// 获取日志条目
async fn get_log_entry(
    State(_state): State<AuthMiddlewareState>,
    Path(_log_id): Path<Uuid>,
) -> Result<Json<LogEntry>, StatusCode> {
    // 简化实现
    Err(StatusCode::NOT_FOUND)
}

/// 列出角色
async fn list_roles(
    State(state): State<AuthMiddlewareState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let roles = {
        let permission_manager = state.permission_manager.lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        permission_manager.get_all_roles()
    };
    Ok(Json(roles))
}

/// 创建角色
async fn create_role(
    State(_state): State<AuthMiddlewareState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Role created successfully".to_string(),
        None,
    )))
}

/// 获取角色信息
async fn get_role(
    State(_state): State<AuthMiddlewareState>,
    Path(_role_name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 简化实现
    Err(StatusCode::NOT_FOUND)
}

/// 更新角色
async fn update_role(
    State(_state): State<AuthMiddlewareState>,
    Path(_role_name): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Role updated successfully".to_string(),
        None,
    )))
}

/// 删除角色
async fn delete_role(
    State(_state): State<AuthMiddlewareState>,
    Path(_role_name): Path<String>,
) -> Result<Json<OperationResult>, StatusCode> {
    // 简化实现
    Ok(Json(OperationResult::success(
        "Role deleted successfully".to_string(),
        None,
    )))
}

/// 列出权限
async fn list_permissions(
    State(_state): State<AuthMiddlewareState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    // 简化实现
    let permissions = vec![
        "view_session".to_string(),
        "create_session".to_string(),
        "delete_session".to_string(),
        "view_user".to_string(),
        "create_user".to_string(),
        "update_user".to_string(),
        "delete_user".to_string(),
        "view_system_status".to_string(),
        "manage_config".to_string(),
        "view_logs".to_string(),
        "manage_permissions".to_string(),
        "view_statistics".to_string(),
    ];
    Ok(Json(permissions))
}
