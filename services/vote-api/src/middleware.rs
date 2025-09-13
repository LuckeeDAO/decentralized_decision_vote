use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};

/// Request logging middleware
#[allow(dead_code)]
pub async fn logging_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    info!(
        "{} {} - User-Agent: {}",
        method,
        uri,
        user_agent
    );
    
    let response = next.run(request).await;
    
    let status = response.status();
    if status.is_client_error() {
        warn!("Client error: {}", status);
    } else if status.is_server_error() {
        warn!("Server error: {}", status);
    }
    
    Ok(response)
}

/// CORS middleware
#[allow(dead_code)]
pub async fn cors_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let origin = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("*");
    
    let mut response = next.run(request).await;
    
    // Add CORS headers
    response.headers_mut().insert(
        "access-control-allow-origin",
        HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*")),
    );
    
    response.headers_mut().insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    
    response.headers_mut().insert(
        "access-control-allow-headers",
        HeaderValue::from_static("Content-Type, Authorization"),
    );
    
    Ok(response)
}

/// Rate limiting middleware (placeholder)
#[allow(dead_code)]
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement actual rate limiting
    // For now, just pass through
    Ok(next.run(request).await)
}
