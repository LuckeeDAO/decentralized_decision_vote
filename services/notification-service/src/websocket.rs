//! WebSocket server for real-time notifications

use crate::{NotificationMessage, NotificationError};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use serde_json;
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// WebSocket连接信息
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: Uuid,
    pub recipient: String,
    pub sender: tokio::sync::mpsc::UnboundedSender<NotificationMessage>,
}

/// WebSocket服务器状态
#[derive(Debug)]
pub struct WebSocketState {
    pub connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    pub event_sender: broadcast::Sender<NotificationMessage>,
}

impl WebSocketState {
    pub fn new(event_sender: broadcast::Sender<NotificationMessage>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub async fn add_connection(&self, recipient: String, connection: WebSocketConnection) {
        let mut connections = self.connections.write().await;
        connections.insert(recipient.clone(), connection);
        info!("Added WebSocket connection for recipient: {}", recipient);
    }

    pub async fn remove_connection(&self, recipient: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(recipient).is_some() {
            info!("Removed WebSocket connection for recipient: {}", recipient);
        }
    }

    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    pub async fn send_to_recipient(&self, recipient: &str, message: NotificationMessage) -> Result<(), NotificationError> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(recipient) {
            if let Err(e) = connection.sender.send(message) {
                error!("Failed to send message to WebSocket connection: {}", e);
                return Err(NotificationError::WebSocket(format!("Failed to send message: {}", e)));
            }
            Ok(())
        } else {
            Err(NotificationError::WebSocket(format!("No connection found for recipient: {}", recipient)))
        }
    }
}

/// 创建WebSocket路由
pub fn create_websocket_router(state: WebSocketState) -> Router {
    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state)
}

/// WebSocket处理器
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

/// 处理WebSocket连接
async fn websocket_connection(socket: WebSocket, state: WebSocketState) {
    let (mut sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();
    
    // 创建消息通道
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<NotificationMessage>();
    
    // 创建发送者通道用于从接收任务向发送任务传递消息
    let (sender_tx, mut sender_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    
    // 接收客户端消息的任务
    let state_clone = state.clone();
    let connection_id_clone = connection_id;
    let recipient_tx = tx.clone();
    let sender_tx_clone = sender_tx.clone();
    
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    
                    // 解析消息
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(recipient) = data.get("recipient").and_then(|v| v.as_str()) {
                            // 注册连接
                            let connection = WebSocketConnection {
                                id: connection_id_clone,
                                recipient: recipient.to_string(),
                                sender: recipient_tx.clone(),
                            };
                            state_clone.add_connection(recipient.to_string(), connection).await;
                            
                            // 发送确认消息
                            let ack = serde_json::json!({
                                "type": "connection_established",
                                "connection_id": connection_id_clone,
                                "recipient": recipient
                            });
                            
                            if let Ok(ack_text) = serde_json::to_string(&ack) {
                                if let Err(e) = sender_tx_clone.send(Message::Text(ack_text)) {
                                    error!("Failed to queue acknowledgment: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = sender_tx_clone.send(Message::Pong(data)) {
                        error!("Failed to queue pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // 忽略pong消息
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary message, ignoring");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // 发送消息到客户端的任务
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // 处理来自接收任务的消息（确认、pong等）
                msg = sender_rx.recv() => {
                    match msg {
                        Some(message) => {
                            if let Err(e) = sender.send(message).await {
                                error!("Failed to send message: {}", e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
                // 处理通知消息
                notification = rx.recv() => {
                    match notification {
                        Some(message) => {
                            let notification_json = serde_json::json!({
                                "type": "notification",
                                "id": message.id,
                                "notification_type": message.notification_type,
                                "priority": message.priority,
                                "title": message.title,
                                "content": message.content,
                                "metadata": message.metadata,
                                "timestamp": message.created_at
                            });

                            if let Ok(notification_text) = serde_json::to_string(&notification_json) {
                                if let Err(e) = sender.send(Message::Text(notification_text)).await {
                                    error!("Failed to send notification: {}", e);
                                    break;
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // 等待任务完成
    tokio::select! {
        _ = receive_task => {
            info!("WebSocket receive task completed");
        }
        _ = send_task => {
            info!("WebSocket send task completed");
        }
    }

    // 清理连接
    // 注意：这里需要从连接映射中移除，但由于我们不知道recipient，
    // 实际实现中应该维护一个连接ID到recipient的映射
    info!("WebSocket connection {} closed", connection_id);
}

/// WebSocket服务器
pub struct WebSocketServer {
    state: WebSocketState,
    router: Router,
}

impl WebSocketServer {
    pub fn new(event_sender: broadcast::Sender<NotificationMessage>) -> Self {
        let state = WebSocketState::new(event_sender);
        let router = create_websocket_router(state.clone());
        
        Self { state, router }
    }

    pub fn get_router(self) -> Router {
        self.router
    }

    pub fn get_router_ref(&self) -> &Router {
        &self.router
    }

    pub async fn get_connection_count(&self) -> usize {
        self.state.get_connection_count().await
    }

    pub async fn send_to_recipient(&self, recipient: &str, message: NotificationMessage) -> Result<(), NotificationError> {
        self.state.send_to_recipient(recipient, message).await
    }

    pub fn get_state(&self) -> &WebSocketState {
        &self.state
    }
}

// 为WebSocketState实现Clone
impl Clone for WebSocketState {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            event_sender: self.event_sender.clone(),
        }
    }
}
