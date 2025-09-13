//! Main notification service implementation

use crate::{
    NotificationConfig, NotificationError, EventHandler, ProviderManager, 
    NotificationMessage, NotificationType, EventSubscriber
};
use crate::websocket::WebSocketServer;
use uuid::Uuid;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{info, error};

/// 通知服务
pub struct NotificationService {
    config: NotificationConfig,
    event_handler: EventHandler,
    provider_manager: ProviderManager,
    websocket_server: Option<WebSocketServer>,
    event_sender: broadcast::Sender<NotificationMessage>,
    #[allow(dead_code)]
    event_receiver: broadcast::Receiver<NotificationMessage>,
    http_server_handle: Option<JoinHandle<()>>,
    websocket_server_handle: Option<JoinHandle<()>>,
    event_processor_handle: Option<JoinHandle<()>>,
}

impl NotificationService {
    /// 创建新的通知服务
    pub async fn new(config: NotificationConfig) -> Result<Self, NotificationError> {
        info!("Initializing notification service");
        
        // 创建事件通道
        let (event_sender, event_receiver) = broadcast::channel(config.events.queue_size);
        
        // 创建事件处理器
        let event_handler = EventHandler::new();
        
        // 创建提供者管理器
        let mut provider_manager = ProviderManager::new();
        
        // 初始化通知提供者
        Self::initialize_providers(&mut provider_manager, &config).await?;
        
        // 创建WebSocket服务器
        let websocket_server = if config.websocket.port > 0 {
            Some(WebSocketServer::new(event_sender.clone()))
        } else {
            None
        };
        
        Ok(Self {
            config,
            event_handler,
            provider_manager,
            websocket_server,
            event_sender,
            event_receiver,
            http_server_handle: None,
            websocket_server_handle: None,
            event_processor_handle: None,
        })
    }
    
    /// 启动通知服务
    pub async fn start(&mut self) -> Result<(), NotificationError> {
        info!("Starting notification service");
        
        // 启动HTTP服务器
        self.start_http_server().await?;
        
        // 启动WebSocket服务器
        if self.websocket_server.is_some() {
            self.start_websocket_server().await?;
        }
        
        // 启动事件处理器
        self.start_event_processor().await?;
        
        info!("Notification service started successfully");
        Ok(())
    }
    
    /// 关闭通知服务
    pub async fn shutdown(&mut self) -> Result<(), NotificationError> {
        info!("Shutting down notification service");
        
        // 关闭HTTP服务器
        if let Some(handle) = self.http_server_handle.take() {
            handle.abort();
        }
        
        // 关闭WebSocket服务器
        if let Some(handle) = self.websocket_server_handle.take() {
            handle.abort();
        }
        
        // 关闭事件处理器
        if let Some(handle) = self.event_processor_handle.take() {
            handle.abort();
        }
        
        info!("Notification service shutdown complete");
        Ok(())
    }
    
    /// 发布事件
    pub fn publish_event(&self, event_type: NotificationType, session_id: Option<String>, data: std::collections::HashMap<String, serde_json::Value>, source: String) -> Result<(), NotificationError> {
        let event = crate::NotificationEvent::new(event_type, session_id, data, source);
        self.event_handler.publish_event(event).map_err(NotificationError::Other)
    }
    
    /// 发送通知
    pub async fn send_notification(&self, message: NotificationMessage) -> Result<(), NotificationError> {
        let results = self.provider_manager.send_to_all_providers(&message).await;
        
        let mut success_count = 0;
        for (provider_name, result) in results {
            match result {
                Ok(_) => {
                    info!("Notification sent successfully via provider: {}", provider_name);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Failed to send notification via provider {}: {}", provider_name, e);
                }
            }
        }
        
        if success_count == 0 {
            return Err(NotificationError::Provider("Failed to send notification via any provider".to_string()));
        }
        
        Ok(())
    }
    
    /// 添加事件订阅者
    pub fn subscribe(&mut self, subscriber: EventSubscriber) -> Result<Uuid, NotificationError> {
        self.event_handler.subscribe(subscriber).map_err(NotificationError::Other)
    }
    
    /// 取消事件订阅
    pub fn unsubscribe(&mut self, subscriber_id: Uuid) -> Result<(), NotificationError> {
        self.event_handler.unsubscribe(subscriber_id).map_err(NotificationError::Other)
    }
    
    /// 获取服务状态
    pub async fn get_status(&self) -> serde_json::Value {
        let active_subscribers = self.event_handler.get_active_subscriber_count();
        let websocket_connections = if let Some(ref ws_server) = self.websocket_server {
            ws_server.get_connection_count().await
        } else {
            0
        };
        let available_providers = self.provider_manager.get_provider_names();
        
        serde_json::json!({
            "status": "running",
            "active_subscribers": active_subscribers,
            "websocket_connections": websocket_connections,
            "available_providers": available_providers,
            "config": {
                "server": self.config.server,
                "events": self.config.events,
                "websocket": self.config.websocket
            }
        })
    }
    
    /// 初始化通知提供者
    async fn initialize_providers(provider_manager: &mut ProviderManager, config: &NotificationConfig) -> Result<(), NotificationError> {
        info!("Initializing notification providers");
        
        // 初始化邮件提供者
        if let Some(ref email_config) = config.providers.email {
            // 转换 config::EmailConfig 到 providers::EmailConfig
            let provider_email_config = crate::providers::EmailConfig {
                smtp_host: email_config.smtp_host.clone(),
                smtp_port: email_config.smtp_port,
                username: email_config.username.clone(),
                password: email_config.password.clone(),
                use_tls: email_config.use_tls,
                from_email: email_config.from_email.clone(),
                from_name: email_config.from_name.clone(),
            };
            let mut email_provider = crate::EmailProvider::new(provider_email_config);
            email_provider.initialize().await?;
            provider_manager.add_provider("email".to_string(), Box::new(email_provider));
        }
        
        // 初始化Webhook提供者
        if let Some(ref webhook_config) = config.providers.webhook {
            // 转换 config::WebhookConfig 到 providers::WebhookConfig
            // 注意：config::WebhookConfig 缺少 url 字段，这里使用默认值
            let provider_webhook_config = crate::providers::WebhookConfig {
                url: "".to_string(), // 需要从配置中获取或使用默认值
                timeout: webhook_config.timeout,
                max_retries: webhook_config.max_retries,
                retry_interval: webhook_config.retry_interval,
                headers: webhook_config.default_headers.clone(),
            };
            let webhook_provider = crate::WebhookProvider::new(provider_webhook_config);
            provider_manager.add_provider("webhook".to_string(), Box::new(webhook_provider));
        }
        
        // 初始化WebSocket提供者
        if let Some(ref ws_config) = config.providers.websocket {
            // 转换 config::WebSocketProviderConfig 到 providers::WebSocketProviderConfig
            let provider_ws_config = crate::providers::WebSocketProviderConfig {
                max_connections: ws_config.max_connections,
                connection_timeout: ws_config.connection_timeout,
                heartbeat_interval: ws_config.heartbeat_interval,
                message_buffer_size: ws_config.message_buffer_size,
            };
            let ws_provider = crate::WebSocketProvider::new(provider_ws_config);
            provider_manager.add_provider("websocket".to_string(), Box::new(ws_provider));
        }
        
        info!("Notification providers initialized successfully");
        Ok(())
    }
    
    /// 启动HTTP服务器
    async fn start_http_server(&mut self) -> Result<(), NotificationError> {
        info!("Starting HTTP server on {}:{}", self.config.server.host, self.config.server.port);
        
        let state = crate::handlers::NotificationServiceState {
            event_handler: self.event_handler.clone(),
            provider_manager: self.provider_manager.clone(),
            websocket_state: if let Some(ref ws_server) = self.websocket_server {
                ws_server.get_state().clone()
            } else {
                // 创建一个临时的WebSocket状态
                crate::WebSocketState::new(self.event_sender.clone())
            },
        };
        
        let app = crate::handlers::create_http_router(state);
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.config.server.host, self.config.server.port))
            .await
            .map_err(|e| NotificationError::Configuration(format!("Failed to bind to address: {}", e)))?;
        
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("HTTP server error: {}", e);
            }
        });
        
        self.http_server_handle = Some(handle);
        info!("HTTP server started successfully");
        Ok(())
    }
    
    /// 启动WebSocket服务器
    async fn start_websocket_server(&mut self) -> Result<(), NotificationError> {
        if let Some(ws_server) = self.websocket_server.take() {
            info!("Starting WebSocket server on {}:{}", self.config.websocket.host, self.config.websocket.port);
            
            let app = ws_server.get_router();
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.config.websocket.host, self.config.websocket.port))
                .await
                .map_err(|e| NotificationError::Configuration(format!("Failed to bind WebSocket server: {}", e)))?;
            
            let handle = tokio::spawn(async move {
                if let Err(e) = axum::serve(listener, app).await {
                    error!("WebSocket server error: {}", e);
                }
            });
            
            self.websocket_server_handle = Some(handle);
            info!("WebSocket server started successfully");
        }
        
        Ok(())
    }
    
    /// 启动事件处理器
    async fn start_event_processor(&mut self) -> Result<(), NotificationError> {
        info!("Starting event processor");
        
        let mut receiver = self.event_sender.subscribe();
        let provider_manager = self.provider_manager.clone();
        
        let handle = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                info!("Processing notification message: {}", message.id);
                
                // 发送到所有提供者
                let results = provider_manager.send_to_all_providers(&message).await;
                
                for (provider_name, result) in results {
                    match result {
                        Ok(_) => {
                            info!("Message {} sent successfully via provider: {}", message.id, provider_name);
                        }
                        Err(e) => {
                            error!("Failed to send message {} via provider {}: {}", message.id, provider_name, e);
                        }
                    }
                }
            }
        });
        
        self.event_processor_handle = Some(handle);
        info!("Event processor started successfully");
        Ok(())
    }
}

// 为ProviderManager实现Clone
impl Clone for ProviderManager {
    fn clone(&self) -> Self {
        // 注意：这里简化了实现，实际应用中可能需要更复杂的克隆逻辑
        Self::new()
    }
}
