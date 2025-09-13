//! Main admin API service implementation

use crate::{
    AdminConfig, AdminError, AuthService, PermissionManager,
    middleware::AuthMiddlewareState,
    handlers::create_http_router,
};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tracing::{info, error};

/// 管理API服务
pub struct AdminApiService {
    config: AdminConfig,
    auth_service: Arc<AuthService>,
    permission_manager: Arc<Mutex<PermissionManager>>,
    http_server_handle: Option<JoinHandle<()>>,
}

impl AdminApiService {
    /// 创建新的管理API服务
    pub async fn new(config: AdminConfig) -> Result<Self, AdminError> {
        info!("Initializing admin API service");
        
        // 创建认证服务
        let auth_service = Arc::new(AuthService::new(
            config.auth.jwt_secret.clone(),
            config.auth.jwt_expiry_hours,
        ));
        
        // 创建权限管理器
        let permission_manager = Arc::new(Mutex::new(PermissionManager::new()));
        
        Ok(Self {
            config,
            auth_service,
            permission_manager,
            http_server_handle: None,
        })
    }
    
    /// 启动管理API服务
    pub async fn start(&mut self) -> Result<(), AdminError> {
        info!("Starting admin API service");
        
        // 启动HTTP服务器
        self.start_http_server().await?;
        
        info!("Admin API service started successfully");
        Ok(())
    }
    
    /// 关闭管理API服务
    pub async fn shutdown(&mut self) -> Result<(), AdminError> {
        info!("Shutting down admin API service");
        
        // 关闭HTTP服务器
        if let Some(handle) = self.http_server_handle.take() {
            handle.abort();
        }
        
        info!("Admin API service shutdown complete");
        Ok(())
    }
    
    /// 获取服务状态
    pub async fn get_status(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "running",
            "config": {
                "server": self.config.server,
                "auth": {
                    "jwt_expiry_hours": self.config.auth.jwt_expiry_hours,
                    "min_password_length": self.config.auth.min_password_length,
                },
                "permissions": {
                    "cache_ttl": self.config.permissions.cache_ttl,
                }
            }
        })
    }
    
    /// 启动HTTP服务器
    async fn start_http_server(&mut self) -> Result<(), AdminError> {
        info!("Starting HTTP server on {}:{}", self.config.server.host, self.config.server.port);
        
        let middleware_state = AuthMiddlewareState {
            auth_service: Arc::clone(&self.auth_service),
            permission_manager: Arc::clone(&self.permission_manager),
        };
        
        let app = create_http_router(middleware_state);
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.config.server.host, self.config.server.port))
            .await
            .map_err(|e| AdminError::Configuration(format!("Failed to bind to address: {}", e)))?;
        
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("HTTP server error: {}", e);
            }
        });
        
        self.http_server_handle = Some(handle);
        info!("HTTP server started successfully");
        Ok(())
    }
}
