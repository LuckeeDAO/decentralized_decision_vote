//! Admin API Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 管理API配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 认证配置
    pub auth: AuthConfig,
    /// 权限配置
    pub permissions: PermissionsConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
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
    /// 最大请求体大小（MB）
    pub max_body_size: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8081,
            workers: 4,
            request_timeout: 30,
            max_body_size: 10,
        }
    }
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT密钥
    pub jwt_secret: String,
    /// JWT过期时间（小时）
    pub jwt_expiry_hours: u64,
    /// 刷新令牌过期时间（天）
    pub refresh_token_expiry_days: u64,
    /// 密码最小长度
    pub min_password_length: usize,
    /// 密码复杂度要求
    pub password_complexity: PasswordComplexityConfig,
    /// 登录失败锁定配置
    pub lockout: LockoutConfig,
}

/// 密码复杂度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordComplexityConfig {
    /// 需要大写字母
    pub require_uppercase: bool,
    /// 需要小写字母
    pub require_lowercase: bool,
    /// 需要数字
    pub require_numbers: bool,
    /// 需要特殊字符
    pub require_special_chars: bool,
}

impl Default for PasswordComplexityConfig {
    fn default() -> Self {
        Self {
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
        }
    }
}

/// 登录锁定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutConfig {
    /// 最大失败尝试次数
    pub max_attempts: u32,
    /// 锁定时间（分钟）
    pub lockout_duration_minutes: u64,
    /// 是否启用锁定
    pub enabled: bool,
}

impl Default for LockoutConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_minutes: 15,
            enabled: true,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-here".to_string(),
            jwt_expiry_hours: 24,
            refresh_token_expiry_days: 7,
            min_password_length: 8,
            password_complexity: PasswordComplexityConfig::default(),
            lockout: LockoutConfig::default(),
        }
    }
}

/// 权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    /// 默认角色权限
    pub default_roles: HashMap<String, Vec<String>>,
    /// 权限继承
    pub inheritance: HashMap<String, Vec<String>>,
    /// 权限缓存时间（秒）
    pub cache_ttl: u64,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        let mut default_roles = HashMap::new();
        default_roles.insert("admin".to_string(), vec![
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
        ]);
        default_roles.insert("moderator".to_string(), vec![
            "view_session".to_string(),
            "view_user".to_string(),
            "view_system_status".to_string(),
            "view_logs".to_string(),
            "view_statistics".to_string(),
        ]);
        default_roles.insert("viewer".to_string(), vec![
            "view_session".to_string(),
            "view_system_status".to_string(),
            "view_statistics".to_string(),
        ]);

        Self {
            default_roles,
            inheritance: HashMap::new(),
            cache_ttl: 300,
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 查询超时时间（秒）
    pub query_timeout: u64,
    /// 是否启用连接池
    pub enable_pooling: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:./data/admin.db".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            query_timeout: 60,
            enable_pooling: true,
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
    /// 是否记录访问日志
    pub access_log: bool,
    /// 是否记录操作日志
    pub operation_log: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_output: true,
            file_path: "./logs/admin-api.log".to_string(),
            access_log: true,
            operation_log: true,
        }
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// 是否启用HTTPS
    pub enable_https: bool,
    /// SSL证书路径
    pub ssl_cert_path: Option<String>,
    /// SSL私钥路径
    pub ssl_key_path: Option<String>,
    /// CORS配置
    pub cors: CorsConfig,
    /// 速率限制配置
    pub rate_limit: RateLimitConfig,
    /// 请求头安全配置
    pub headers: SecurityHeadersConfig,
}

/// CORS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 允许的源
    pub allowed_origins: Vec<String>,
    /// 允许的方法
    pub allowed_methods: Vec<String>,
    /// 允许的头部
    pub allowed_headers: Vec<String>,
    /// 是否允许凭据
    pub allow_credentials: bool,
    /// 预检请求缓存时间（秒）
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
            ],
            allow_credentials: true,
            max_age: 86400,
        }
    }
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用速率限制
    pub enabled: bool,
    /// 每分钟最大请求数
    pub requests_per_minute: u32,
    /// 每小时最大请求数
    pub requests_per_hour: u32,
    /// 速率限制键前缀
    pub key_prefix: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 100,
            requests_per_hour: 1000,
            key_prefix: "admin_api_rate_limit".to_string(),
        }
    }
}

/// 安全头部配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeadersConfig {
    /// 是否启用安全头部
    pub enabled: bool,
    /// 自定义头部
    pub custom_headers: HashMap<String, String>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        let mut custom_headers = HashMap::new();
        custom_headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        custom_headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        custom_headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        custom_headers.insert("Strict-Transport-Security".to_string(), "max-age=31536000".to_string());

        Self {
            enabled: true,
            custom_headers,
        }
    }
}


/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 是否启用健康检查
    pub health_check: bool,
    /// 健康检查路径
    pub health_check_path: String,
    /// 是否启用指标收集
    pub metrics: bool,
    /// 指标收集路径
    pub metrics_path: String,
    /// 是否启用性能监控
    pub performance_monitoring: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check: true,
            health_check_path: "/health".to_string(),
            metrics: true,
            metrics_path: "/metrics".to_string(),
            performance_monitoring: true,
        }
    }
}

