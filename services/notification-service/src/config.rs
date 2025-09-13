//! Notification Service Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 通知服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 服务配置
    pub server: ServerConfig,
    /// 通知提供者配置
    pub providers: ProvidersConfig,
    /// 事件订阅配置
    pub events: EventsConfig,
    /// WebSocket配置
    pub websocket: WebSocketConfig,
    /// 重试配置
    pub retry: RetryConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 工作线程数
    pub workers: usize,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8082,
            workers: 4,
            request_timeout: 30,
        }
    }
}

/// 通知提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    /// 邮件配置
    pub email: Option<EmailConfig>,
    /// Webhook配置
    pub webhook: Option<WebhookConfig>,
    /// WebSocket配置
    pub websocket: Option<WebSocketProviderConfig>,
    /// 默认提供者列表
    pub default_providers: Vec<String>,
}

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP服务器地址
    pub smtp_host: String,
    /// SMTP端口
    pub smtp_port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否使用TLS
    pub use_tls: bool,
    /// 发件人邮箱
    pub from_email: String,
    /// 发件人名称
    pub from_name: String,
}

/// Webhook配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// 默认超时时间（秒）
    pub timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（秒）
    pub retry_interval: u64,
    /// 自定义headers
    pub default_headers: HashMap<String, String>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            max_retries: 3,
            retry_interval: 5,
            default_headers: HashMap::new(),
        }
    }
}

/// WebSocket提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketProviderConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 消息缓冲区大小
    pub message_buffer_size: usize,
}

impl Default for WebSocketProviderConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: 30,
            heartbeat_interval: 30,
            message_buffer_size: 1000,
        }
    }
}

/// 事件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsConfig {
    /// 事件队列大小
    pub queue_size: usize,
    /// 事件处理线程数
    pub worker_threads: usize,
    /// 事件持久化
    pub persistence: EventPersistenceConfig,
    /// 事件过滤
    pub filtering: EventFilteringConfig,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            queue_size: 10000,
            worker_threads: 4,
            persistence: EventPersistenceConfig::default(),
            filtering: EventFilteringConfig::default(),
        }
    }
}

/// 事件持久化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPersistenceConfig {
    /// 是否启用持久化
    pub enabled: bool,
    /// 存储路径
    pub storage_path: String,
    /// 保留时间（天）
    pub retention_days: u32,
    /// 批量大小
    pub batch_size: usize,
}

impl Default for EventPersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage_path: "./data/events".to_string(),
            retention_days: 30,
            batch_size: 100,
        }
    }
}

/// 事件过滤配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilteringConfig {
    /// 是否启用过滤
    pub enabled: bool,
    /// 过滤规则
    pub rules: Vec<FilterRule>,
}

impl Default for EventFilteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
        }
    }
}

/// 过滤规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// 规则名称
    pub name: String,
    /// 事件类型
    pub event_type: String,
    /// 条件
    pub condition: String,
    /// 动作
    pub action: FilterAction,
}

/// 过滤动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
    /// 修改
    Modify,
}

/// WebSocket配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 路径
    pub path: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8083,
            path: "/ws".to_string(),
            max_connections: 1000,
            connection_timeout: 30,
            heartbeat_interval: 30,
        }
    }
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试间隔（秒）
    pub initial_interval: u64,
    /// 最大重试间隔（秒）
    pub max_interval: u64,
    /// 重试间隔倍数
    pub multiplier: f64,
    /// 随机化因子
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: 1,
            max_interval: 60,
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式
    pub format: String,
    /// 是否输出到文件
    pub file_output: bool,
    /// 日志文件路径
    pub file_path: String,
    /// 日志轮转
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_output: true,
            file_path: "./logs/notification-service.log".to_string(),
            rotation: LogRotationConfig::default(),
        }
    }
}

/// 日志轮转配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// 最大文件大小（MB）
    pub max_size_mb: u64,
    /// 保留文件数
    pub max_files: u32,
    /// 轮转时间
    pub rotation_time: String,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 100,
            max_files: 5,
            rotation_time: "daily".to_string(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            providers: ProvidersConfig {
                email: None,
                webhook: Some(WebhookConfig::default()),
                websocket: Some(WebSocketProviderConfig::default()),
                default_providers: vec!["websocket".to_string()],
            },
            events: EventsConfig::default(),
            websocket: WebSocketConfig::default(),
            retry: RetryConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}
