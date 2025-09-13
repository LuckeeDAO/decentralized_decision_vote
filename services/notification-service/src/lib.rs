//! Notification Service
//! 
//! 提供多种通知方式和事件订阅功能

pub mod config;
pub mod service;
pub mod handlers;
pub mod providers;
pub mod events;
pub mod websocket;

pub use config::NotificationConfig;
pub use service::NotificationService;
pub use events::{NotificationEvent, EventHandler};
pub use providers::{NotificationProvider, EmailProvider, WebhookProvider, WebSocketProvider, ProviderManager};
pub use websocket::WebSocketState;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    /// 会话创建通知
    SessionCreated,
    /// 承诺提交通知
    CommitmentSubmitted,
    /// 揭示阶段开始通知
    RevealPhaseStarted,
    /// 揭示完成通知
    RevealCompleted,
    /// 结果生成通知
    ResultGenerated,
    /// 系统错误通知
    SystemError,
    /// 自定义通知
    Custom(String),
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationType::SessionCreated => write!(f, "SessionCreated"),
            NotificationType::CommitmentSubmitted => write!(f, "CommitmentSubmitted"),
            NotificationType::RevealPhaseStarted => write!(f, "RevealPhaseStarted"),
            NotificationType::RevealCompleted => write!(f, "RevealCompleted"),
            NotificationType::ResultGenerated => write!(f, "ResultGenerated"),
            NotificationType::SystemError => write!(f, "SystemError"),
            NotificationType::Custom(custom_type) => write!(f, "Custom({})", custom_type),
        }
    }
}

/// 通知优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// 通知状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub content: String,
    pub recipient: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: NotificationStatus,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl NotificationMessage {
    pub fn new(
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: String,
        content: String,
        recipient: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            notification_type,
            priority,
            title,
            content,
            recipient,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            status: NotificationStatus::Pending,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// 事件订阅者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscriber {
    pub id: Uuid,
    pub name: String,
    pub event_types: Vec<NotificationType>,
    pub notification_providers: Vec<String>,
    pub filters: HashMap<String, serde_json::Value>,
    pub active: bool,
}

impl EventSubscriber {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            event_types: Vec::new(),
            notification_providers: Vec::new(),
            filters: HashMap::new(),
            active: true,
        }
    }

    pub fn subscribe_to_event(mut self, event_type: NotificationType) -> Self {
        if !self.event_types.contains(&event_type) {
            self.event_types.push(event_type);
        }
        self
    }

    pub fn add_provider(mut self, provider: String) -> Self {
        if !self.notification_providers.contains(&provider) {
            self.notification_providers.push(provider);
        }
        self
    }

    pub fn with_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.filters.insert(key, value);
        self
    }
}

/// 通知服务错误
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Event subscription error: {0}")]
    EventSubscription(String),
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
    
    #[error("SMTP transport error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
    
    #[error("Address error: {0}")]
    Address(#[from] lettre::address::AddressError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
