//! Notification providers implementation

use crate::{NotificationMessage, NotificationError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use lettre::AsyncTransport;

/// 通知提供者 trait
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// 提供者名称
    fn name(&self) -> &str;
    
    /// 发送通知
    async fn send_notification(&self, message: &NotificationMessage) -> Result<(), NotificationError>;
    
    /// 检查提供者是否可用
    async fn is_available(&self) -> bool;
    
    /// 获取提供者配置
    fn get_config(&self) -> &dyn std::fmt::Debug;
}

/// 邮件通知提供者
pub struct EmailProvider {
    config: EmailConfig,
    client: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub from_email: String,
    pub from_name: String,
}

#[async_trait]
impl NotificationProvider for EmailProvider {
    fn name(&self) -> &str {
        "email"
    }

    async fn send_notification(&self, message: &NotificationMessage) -> Result<(), NotificationError> {
        info!("Sending email notification to: {}", message.recipient);
        
        // 创建邮件消息
        let email = lettre::Message::builder()
            .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
            .to(message.recipient.parse()?)
            .subject(&message.title)
            .body(message.content.clone())?;

        // 发送邮件
        if let Some(ref client) = self.client {
            client.send(email).await?;
            info!("Email notification sent successfully to: {}", message.recipient);
        } else {
            return Err(NotificationError::Provider("Email client not initialized".to_string()));
        }

        Ok(())
    }

    async fn is_available(&self) -> bool {
        self.client.is_some()
    }

    fn get_config(&self) -> &dyn std::fmt::Debug {
        &self.config
    }
}

impl EmailProvider {
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), NotificationError> {
        info!("Initializing email provider");
        
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let mailer = if self.config.use_tls {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&self.config.smtp_host)?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        } else {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&self.config.smtp_host)
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        };

        self.client = Some(mailer);
        info!("Email provider initialized successfully");
        Ok(())
    }
}

/// Webhook通知提供者
pub struct WebhookProvider {
    config: WebhookConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub retry_interval: u64,
    pub headers: HashMap<String, String>,
}

#[async_trait]
impl NotificationProvider for WebhookProvider {
    fn name(&self) -> &str {
        "webhook"
    }

    async fn send_notification(&self, message: &NotificationMessage) -> Result<(), NotificationError> {
        info!("Sending webhook notification to: {}", self.config.url);
        
        let mut request = self.client
            .post(&self.config.url)
            .timeout(std::time::Duration::from_secs(self.config.timeout))
            .json(message);

        // 添加自定义headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        let mut retry_count = 0;
        while retry_count <= self.config.max_retries {
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Webhook notification sent successfully to: {}", self.config.url);
                        return Ok(());
                    } else {
                        warn!("Webhook request failed with status: {}", response.status());
                    }
                }
                Err(e) => {
                    error!("Webhook request failed: {}", e);
                }
            }

            retry_count += 1;
            if retry_count <= self.config.max_retries {
                tokio::time::sleep(std::time::Duration::from_secs(self.config.retry_interval)).await;
            }
        }

        Err(NotificationError::Provider(format!(
            "Failed to send webhook notification after {} retries",
            self.config.max_retries
        )))
    }

    async fn is_available(&self) -> bool {
        // 简单的健康检查
        match self.client.get(&self.config.url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn get_config(&self) -> &dyn std::fmt::Debug {
        &self.config
    }
}

impl WebhookProvider {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

/// WebSocket通知提供者
pub struct WebSocketProvider {
    config: WebSocketProviderConfig,
    connections: HashMap<String, tokio::sync::mpsc::UnboundedSender<NotificationMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketProviderConfig {
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub heartbeat_interval: u64,
    pub message_buffer_size: usize,
}

#[async_trait]
impl NotificationProvider for WebSocketProvider {
    fn name(&self) -> &str {
        "websocket"
    }

    async fn send_notification(&self, message: &NotificationMessage) -> Result<(), NotificationError> {
        info!("Sending WebSocket notification to: {}", message.recipient);
        
        if let Some(sender) = self.connections.get(&message.recipient) {
            if let Err(e) = sender.send(message.clone()) {
                error!("Failed to send WebSocket message: {}", e);
                return Err(NotificationError::WebSocket(format!("Failed to send message: {}", e)));
            }
            info!("WebSocket notification sent successfully to: {}", message.recipient);
        } else {
            warn!("No WebSocket connection found for recipient: {}", message.recipient);
            return Err(NotificationError::WebSocket(format!(
                "No connection found for recipient: {}",
                message.recipient
            )));
        }

        Ok(())
    }

    async fn is_available(&self) -> bool {
        !self.connections.is_empty()
    }

    fn get_config(&self) -> &dyn std::fmt::Debug {
        &self.config
    }
}

impl WebSocketProvider {
    pub fn new(config: WebSocketProviderConfig) -> Self {
        Self {
            config,
            connections: HashMap::new(),
        }
    }

    pub fn add_connection(&mut self, recipient: String, sender: tokio::sync::mpsc::UnboundedSender<NotificationMessage>) {
        let recipient_clone = recipient.clone();
        self.connections.insert(recipient, sender);
        info!("Added WebSocket connection for recipient: {}", recipient_clone);
    }

    pub fn remove_connection(&mut self, recipient: &str) {
        if self.connections.remove(recipient).is_some() {
            info!("Removed WebSocket connection for recipient: {}", recipient);
        }
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn is_at_capacity(&self) -> bool {
        self.connections.len() >= self.config.max_connections
    }
}

/// 通知提供者管理器
pub struct ProviderManager {
    providers: HashMap<String, Box<dyn NotificationProvider>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn add_provider(&mut self, name: String, provider: Box<dyn NotificationProvider>) {
        info!("Adding notification provider: {}", name);
        self.providers.insert(name, provider);
    }

    pub fn remove_provider(&mut self, name: &str) {
        if self.providers.remove(name).is_some() {
            info!("Removed notification provider: {}", name);
        }
    }

    pub fn get_provider(&self, name: &str) -> Option<&dyn NotificationProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn get_provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn send_notification(&self, provider_name: &str, message: &NotificationMessage) -> Result<(), NotificationError> {
        if let Some(provider) = self.providers.get(provider_name) {
            provider.send_notification(message).await
        } else {
            Err(NotificationError::Provider(format!("Provider not found: {}", provider_name)))
        }
    }

    pub async fn send_to_all_providers(&self, message: &NotificationMessage) -> Vec<(String, Result<(), NotificationError>)> {
        let mut results = Vec::new();
        
        for (name, provider) in &self.providers {
            let result = provider.send_notification(message).await;
            results.push((name.clone(), result));
        }
        
        results
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
