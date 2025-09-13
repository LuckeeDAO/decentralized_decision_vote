//! Configuration storage with hot reloading and version management

pub mod store;
pub mod version;
pub mod watcher;
pub mod cache;

pub use store::ConfigStore;
pub use version::{ConfigVersion, VersionManager};
pub use watcher::{ConfigWatcher};
pub use cache::{ConfigCache, CacheStrategy};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 配置项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_sensitive: bool,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: String,
}

impl ConfigItem {
    pub fn new(
        key: String,
        value: serde_json::Value,
        category: String,
        description: Option<String>,
        is_sensitive: bool,
        updated_by: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            key,
            value,
            description,
            category,
            is_sensitive,
            version: 1,
            created_at: now,
            updated_at: now,
            updated_by,
        }
    }

    pub fn update_value(&mut self, value: serde_json::Value, updated_by: String) {
        self.value = value;
        self.version += 1;
        self.updated_at = chrono::Utc::now();
        self.updated_by = updated_by;
    }
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeEvent {
    /// 配置项创建
    Created(ConfigItem),
    /// 配置项更新
    Updated(ConfigItem, ConfigItem), // old, new
    /// 配置项删除
    Deleted(String), // key
    /// 批量更新
    BatchUpdated(Vec<ConfigItem>),
}

/// 配置存储 trait
#[async_trait]
pub trait ConfigStorage: Send + Sync {
    /// 获取配置项
    async fn get(&self, key: &str) -> Result<Option<ConfigItem>, ConfigStoreError>;
    
    /// 设置配置项
    async fn set(&self, item: ConfigItem) -> Result<(), ConfigStoreError>;
    
    /// 删除配置项
    async fn delete(&self, key: &str) -> Result<(), ConfigStoreError>;
    
    /// 获取所有配置项
    async fn get_all(&self) -> Result<Vec<ConfigItem>, ConfigStoreError>;
    
    /// 按类别获取配置项
    async fn get_by_category(&self, category: &str) -> Result<Vec<ConfigItem>, ConfigStoreError>;
    
    /// 批量设置配置项
    async fn set_batch(&self, items: Vec<ConfigItem>) -> Result<(), ConfigStoreError>;
    
    /// 检查配置项是否存在
    async fn exists(&self, key: &str) -> Result<bool, ConfigStoreError>;
}

/// 配置存储错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigStoreError {
    #[error("Configuration not found: {0}")]
    NotFound(String),
    
    #[error("Configuration validation error: {0}")]
    Validation(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
