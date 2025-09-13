//! Middleware for admin API

use crate::{AdminOperation, auth::AuthService, permissions::PermissionManager};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{info, warn, error};
use uuid::Uuid;

/// 认证中间件状态
#[derive(Clone)]
pub struct AuthMiddlewareState {
    pub auth_service: Arc<AuthService>,
    pub permission_manager: Arc<Mutex<PermissionManager>>,
}

/// 用户上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
}

/// 认证中间件
pub async fn auth_middleware(
    State(state): State<AuthMiddlewareState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 提取Authorization头部
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // 检查Bearer token格式
    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid Authorization header format");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // 移除"Bearer "前缀

    // 验证JWT令牌
    let claims = state.auth_service.verify_token(token)
        .map_err(|e| {
            error!("Token verification failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // 检查用户是否存在且活跃
    let user = state.auth_service.get_user(Uuid::parse_str(&claims.sub).unwrap_or_default())
        .ok_or_else(|| {
            warn!("User not found: {}", claims.sub);
            StatusCode::UNAUTHORIZED
        })?;

    if !user.is_active {
        warn!("User account is inactive: {}", user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    // 将用户上下文添加到请求扩展中
    let user_context = UserContext {
        user_id: user.id,
        username: user.username,
        role: user.role,
    };

    let mut request = request;
    request.extensions_mut().insert(user_context);

    Ok(next.run(request).await)
}

/// 权限检查中间件
pub async fn permission_middleware(
    State(state): State<AuthMiddlewareState>,
    operation: AdminOperation,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取用户上下文
    let user_context = request.extensions()
        .get::<UserContext>()
        .ok_or_else(|| {
            error!("User context not found in request");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 检查用户权限
    let has_permission = {
        let mut permission_manager = state.permission_manager.lock()
            .map_err(|_| {
                error!("Failed to acquire permission manager lock");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        permission_manager.check_permission(&user_context.username, &operation)
            .map_err(|e| {
                error!("Permission check failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };

    if !has_permission {
        warn!(
            "User {} does not have permission for operation: {:?}",
            user_context.username, operation
        );
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        "User {} authorized for operation: {:?}",
        user_context.username, operation
    );

    Ok(next.run(request).await)
}

/// 日志中间件
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request.headers()
        .get("User-Agent")
        .and_then(|header| header.to_str().ok())
        .unwrap_or("Unknown");

    // 获取用户信息（如果存在）
    let user_info = request.extensions()
        .get::<UserContext>()
        .map(|ctx| format!("{} ({})", ctx.username, ctx.user_id))
        .unwrap_or_else(|| "Anonymous".to_string());

    info!(
        "Request started: {} {} from {} (User: {})",
        method, uri, user_agent, user_info
    );

    let response = next.run(request).await;
    let duration = start_time.elapsed();

    info!(
        "Request completed: {} {} - Status: {} - Duration: {:?}",
        method, uri, response.status(), duration
    );

    response
}

/// 速率限制中间件
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 简化实现，实际应用中应该使用Redis或其他存储
    // 这里只是示例，实际应该根据IP地址和用户进行限制
    
    let client_ip = request.headers()
        .get("X-Forwarded-For")
        .or_else(|| request.headers().get("X-Real-IP"))
        .and_then(|header| header.to_str().ok())
        .unwrap_or("unknown");

    // 这里应该检查速率限制
    // 为了简化，我们假设总是允许请求
    info!("Rate limit check for IP: {}", client_ip);

    Ok(next.run(request).await)
}

/// CORS中间件
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // 添加CORS头部
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With".parse().unwrap());
    headers.insert("Access-Control-Allow-Credentials", "true".parse().unwrap());
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());

    response
}

/// 安全头部中间件
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // 添加安全头部
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());

    response
}

/// 请求ID中间件
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // 将请求ID添加到请求扩展中
    request.extensions_mut().insert(request_id.clone());
    
    let response = next.run(request).await;
    
    // 将请求ID添加到响应头部
    let mut response = response;
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    
    response
}

/// 错误处理中间件
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Response {
    match next.run(request).await {
        response if response.status().is_success() => response,
        response => {
            let status = response.status();
            let status_code = status.as_u16();
            
            match status_code {
                400 => error!("Bad Request: {}", status),
                401 => error!("Unauthorized: {}", status),
                403 => error!("Forbidden: {}", status),
                404 => error!("Not Found: {}", status),
                500 => error!("Internal Server Error: {}", status),
                _ => warn!("HTTP Error: {}", status),
            }
            
            response
        }
    }
}
