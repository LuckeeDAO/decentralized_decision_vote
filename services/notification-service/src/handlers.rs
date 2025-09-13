//! HTTP handlers for notification service

use crate::{NotificationMessage, EventSubscriber, NotificationType, NotificationPriority};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};
use uuid::Uuid;

/// 通知服务状态
#[derive(Clone)]
pub struct NotificationServiceState {
    pub event_handler: crate::EventHandler,
    pub provider_manager: crate::ProviderManager,
    pub websocket_state: crate::WebSocketState,
}

/// 创建订阅请求
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub event_types: Vec<NotificationType>,
    pub notification_providers: Vec<String>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// 创建订阅响应
#[derive(Debug, Serialize)]
pub struct CreateSubscriptionResponse {
    pub subscriber_id: Uuid,
    pub message: String,
}

/// 发送通知请求
#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub content: String,
    pub recipient: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 发送通知响应
#[derive(Debug, Serialize)]
pub struct SendNotificationResponse {
    pub message_id: Uuid,
    pub message: String,
}

/// 服务状态响应
#[derive(Debug, Serialize)]
pub struct ServiceStatusResponse {
    pub status: String,
    pub active_subscribers: usize,
    pub websocket_connections: usize,
    pub available_providers: Vec<String>,
    pub uptime: String,
}

/// 创建HTTP路由
pub fn create_http_router(state: NotificationServiceState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_service_status))
        .route("/subscriptions", post(create_subscription))
        .route("/subscriptions/:id", delete(delete_subscription))
        .route("/notifications", post(send_notification))
        .route("/subscribers", get(list_subscribers))
        .with_state(state)
}

/// 健康检查
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "notification-service"
    }))
}

/// 获取服务状态
async fn get_service_status(
    State(state): State<NotificationServiceState>,
) -> Result<Json<ServiceStatusResponse>, StatusCode> {
    let active_subscribers = state.event_handler.get_active_subscriber_count();
    let websocket_connections = state.websocket_state.get_connection_count().await;
    let available_providers = state.provider_manager.get_provider_names();
    
    let response = ServiceStatusResponse {
        status: "running".to_string(),
        active_subscribers,
        websocket_connections,
        available_providers,
        uptime: "N/A".to_string(), // 实际实现中应该计算实际运行时间
    };
    
    Ok(Json(response))
}

/// 创建事件订阅
async fn create_subscription(
    State(mut state): State<NotificationServiceState>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<Json<CreateSubscriptionResponse>, StatusCode> {
    info!("Creating subscription for: {}", request.name);
    
    let mut subscriber = EventSubscriber::new(request.name);
    
    // 添加事件类型
    for event_type in request.event_types {
        subscriber = subscriber.subscribe_to_event(event_type);
    }
    
    // 添加通知提供者
    for provider in request.notification_providers {
        subscriber = subscriber.add_provider(provider);
    }
    
    // 添加过滤器
    if let Some(filters) = request.filters {
        for (key, value) in filters {
            subscriber = subscriber.with_filter(key, value);
        }
    }
    
    match state.event_handler.subscribe(subscriber) {
        Ok(subscriber_id) => {
            info!("Successfully created subscription: {}", subscriber_id);
            Ok(Json(CreateSubscriptionResponse {
                subscriber_id,
                message: "Subscription created successfully".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to create subscription: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 删除事件订阅
async fn delete_subscription(
    State(mut state): State<NotificationServiceState>,
    Path(subscriber_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting subscription: {}", subscriber_id);
    
    match state.event_handler.unsubscribe(subscriber_id) {
        Ok(_) => {
            info!("Successfully deleted subscription: {}", subscriber_id);
            Ok(Json(serde_json::json!({
                "message": "Subscription deleted successfully",
                "subscriber_id": subscriber_id
            })))
        }
        Err(e) => {
            error!("Failed to delete subscription: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 发送通知
async fn send_notification(
    State(state): State<NotificationServiceState>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<Json<SendNotificationResponse>, StatusCode> {
    info!("Sending notification to: {}", request.recipient);
    
    let mut message = NotificationMessage::new(
        request.notification_type,
        request.priority,
        request.title,
        request.content,
        request.recipient,
    );
    
    // 添加元数据
    if let Some(metadata) = request.metadata {
        for (key, value) in metadata {
            message = message.with_metadata(key, value);
        }
    }
    
    // 发送到所有可用的提供者
    let results = state.provider_manager.send_to_all_providers(&message).await;
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for (provider_name, result) in results {
        match result {
            Ok(_) => {
                info!("Notification sent successfully via provider: {}", provider_name);
                success_count += 1;
            }
            Err(e) => {
                error!("Failed to send notification via provider {}: {}", provider_name, e);
                failure_count += 1;
            }
        }
    }
    
    if success_count > 0 {
        info!("Notification sent successfully ({} success, {} failures)", success_count, failure_count);
        Ok(Json(SendNotificationResponse {
            message_id: message.id,
            message: format!("Notification sent via {} providers", success_count),
        }))
    } else {
        error!("Failed to send notification via any provider");
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// 列出所有订阅者
async fn list_subscribers(
    State(state): State<NotificationServiceState>,
) -> Result<Json<Vec<EventSubscriber>>, StatusCode> {
    let subscribers = state.event_handler.get_subscribers();
    let owned_subscribers: Vec<EventSubscriber> = subscribers.into_iter().cloned().collect();
    Ok(Json(owned_subscribers))
}
