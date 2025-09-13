//! Admin API Service
//! 
//! 提供管理功能和权限控制

pub mod config;
pub mod service;
pub mod handlers;
pub mod auth;
pub mod permissions;
pub mod middleware;

pub use config::AdminConfig;
pub use service::AdminApiService;
pub use auth::{AuthService, User, Role};
pub use permissions::{Permission, PermissionManager};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 管理操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdminOperation {
    /// 查看会话
    ViewSession,
    /// 创建会话
    CreateSession,
    /// 删除会话
    DeleteSession,
    /// 查看用户
    ViewUser,
    /// 创建用户
    CreateUser,
    /// 更新用户
    UpdateUser,
    /// 删除用户
    DeleteUser,
    /// 查看系统状态
    ViewSystemStatus,
    /// 管理系统配置
    ManageConfig,
    /// 查看日志
    ViewLogs,
    /// 管理权限
    ManagePermissions,
    /// 查看统计信息
    ViewStatistics,
}

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl OperationResult {
    pub fn success(message: String, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message,
            data,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            timestamp: Utc::now(),
        }
    }
}

/// 系统统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatistics {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub total_users: u64,
    pub active_users: u64,
    pub total_commits: u64,
    pub total_reveals: u64,
    pub system_uptime: String,
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkStatistics,
}

/// 网络统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub connections: u32,
    pub requests_per_second: f64,
}

/// 会话管理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementInfo {
    pub session_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub participants: u32,
    pub phase: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 用户管理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserManagementInfo {
    pub user_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// 配置管理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManagementInfo {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_sensitive: bool,
    pub last_updated: DateTime<Utc>,
    pub updated_by: String,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 管理API错误
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
    
    #[error("Boxed error: {0}")]
    Boxed(#[from] Box<dyn std::error::Error>),
}
