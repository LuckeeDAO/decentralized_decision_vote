//! Event storage with query and replay capabilities

pub mod store;
pub mod query;
pub mod replay;
pub mod index;

pub use store::{EventStore, EventStoreError};
pub use query::{EventQuery, QueryBuilder, QueryResult};
pub use replay::{EventReplayer, ReplayOptions, ReplayResult};
pub use index::{EventIndex, IndexManager};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 会话创建
    SessionCreated,
    /// 承诺提交
    CommitmentSubmitted,
    /// 揭示阶段开始
    RevealPhaseStarted,
    /// 揭示完成
    RevealCompleted,
    /// 结果生成
    ResultGenerated,
    /// 系统错误
    SystemError,
    /// 自定义事件
    Custom(String),
}

/// 事件严重级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::SessionCreated => write!(f, "SessionCreated"),
            EventType::CommitmentSubmitted => write!(f, "CommitmentSubmitted"),
            EventType::RevealPhaseStarted => write!(f, "RevealPhaseStarted"),
            EventType::RevealCompleted => write!(f, "RevealCompleted"),
            EventType::ResultGenerated => write!(f, "ResultGenerated"),
            EventType::SystemError => write!(f, "SystemError"),
            EventType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Debug => write!(f, "Debug"),
            EventSeverity::Info => write!(f, "Info"),
            EventSeverity::Warning => write!(f, "Warning"),
            EventSeverity::Error => write!(f, "Error"),
            EventSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub session_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub source: String,
    pub message: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub version: u64,
}

impl Event {
    pub fn new(
        event_type: EventType,
        severity: EventSeverity,
        source: String,
        message: String,
        session_id: Option<String>,
        user_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            severity,
            session_id,
            user_id,
            source,
            message,
            data: HashMap::new(),
            timestamp: Utc::now(),
            correlation_id: None,
            causation_id: None,
            version: 1,
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
}

/// 事件存储 trait
#[async_trait]
pub trait EventStorage: Send + Sync {
    /// 存储事件
    async fn store_event(&self, event: Event) -> Result<(), EventStoreError>;
    
    /// 批量存储事件
    async fn store_events(&self, events: Vec<Event>) -> Result<(), EventStoreError>;
    
    /// 根据ID获取事件
    async fn get_event(&self, event_id: Uuid) -> Result<Option<Event>, EventStoreError>;
    
    /// 根据会话ID获取事件
    async fn get_events_by_session(&self, session_id: &str) -> Result<Vec<Event>, EventStoreError>;
    
    /// 根据用户ID获取事件
    async fn get_events_by_user(&self, user_id: Uuid) -> Result<Vec<Event>, EventStoreError>;
    
    /// 根据时间范围获取事件
    async fn get_events_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Event>, EventStoreError>;
    
    /// 根据事件类型获取事件
    async fn get_events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError>;
    
    /// 获取所有事件
    async fn get_all_events(&self) -> Result<Vec<Event>, EventStoreError>;
    
    /// 删除事件
    async fn delete_event(&self, event_id: Uuid) -> Result<(), EventStoreError>;
    
    /// 清理过期事件
    async fn cleanup_expired_events(&self, before: DateTime<Utc>) -> Result<u64, EventStoreError>;
}

