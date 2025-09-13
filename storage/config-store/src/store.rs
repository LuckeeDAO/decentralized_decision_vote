//! Configuration storage implementations

use crate::{ConfigStorage, ConfigStoreError, ConfigItem};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// 内存配置存储
pub struct MemoryConfigStore {
    configs: Arc<RwLock<HashMap<String, ConfigItem>>>,
}

impl MemoryConfigStore {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ConfigStorage for MemoryConfigStore {
    async fn get(&self, key: &str) -> Result<Option<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.get(key).cloned())
    }

    async fn set(&self, item: ConfigItem) -> Result<(), ConfigStoreError> {
        let mut configs = self.configs.write().await;
        configs.insert(item.key.clone(), item);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), ConfigStoreError> {
        let mut configs = self.configs.write().await;
        configs.remove(key);
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.values().cloned().collect())
    }

    async fn get_by_category(&self, category: &str) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs
            .values()
            .filter(|item| item.category == category)
            .cloned()
            .collect())
    }

    async fn set_batch(&self, items: Vec<ConfigItem>) -> Result<(), ConfigStoreError> {
        let mut configs = self.configs.write().await;
        for item in items {
            configs.insert(item.key.clone(), item);
        }
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.contains_key(key))
    }
}

/// 文件配置存储
pub struct FileConfigStore {
    file_path: PathBuf,
    configs: Arc<RwLock<HashMap<String, ConfigItem>>>,
}

impl FileConfigStore {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(&self) -> Result<(), ConfigStoreError> {
        if !self.file_path.exists() {
            info!("Config file does not exist, creating empty store");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let configs: HashMap<String, ConfigItem> = serde_json::from_str(&content)?;

        let mut store = self.configs.write().await;
        *store = configs;
        
        info!("Loaded {} config items from file", store.len());
        Ok(())
    }

    /// 保存配置到文件
    pub async fn save_to_file(&self) -> Result<(), ConfigStoreError> {
        let configs = self.configs.read().await;
        let content = serde_json::to_string_pretty(&*configs)?;
        
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&self.file_path, content).await?;
        info!("Saved {} config items to file", configs.len());
        Ok(())
    }
}

#[async_trait]
impl ConfigStorage for FileConfigStore {
    async fn get(&self, key: &str) -> Result<Option<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.get(key).cloned())
    }

    async fn set(&self, item: ConfigItem) -> Result<(), ConfigStoreError> {
        {
            let mut configs = self.configs.write().await;
            configs.insert(item.key.clone(), item);
        }
        
        // 保存到文件
        self.save_to_file().await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), ConfigStoreError> {
        {
            let mut configs = self.configs.write().await;
            configs.remove(key);
        }
        
        // 保存到文件
        self.save_to_file().await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.values().cloned().collect())
    }

    async fn get_by_category(&self, category: &str) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs
            .values()
            .filter(|item| item.category == category)
            .cloned()
            .collect())
    }

    async fn set_batch(&self, items: Vec<ConfigItem>) -> Result<(), ConfigStoreError> {
        {
            let mut configs = self.configs.write().await;
            for item in items {
                configs.insert(item.key.clone(), item);
            }
        }
        
        // 保存到文件
        self.save_to_file().await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, ConfigStoreError> {
        let configs = self.configs.read().await;
        Ok(configs.contains_key(key))
    }
}

/// 配置存储管理器
pub struct ConfigStore {
    storage: Box<dyn ConfigStorage>,
    change_sender: tokio::sync::broadcast::Sender<crate::ConfigChangeEvent>,
}

impl ConfigStore {
    pub fn new(storage: Box<dyn ConfigStorage>) -> Self {
        let (sender, _receiver) = tokio::sync::broadcast::channel(1000);
        Self {
            storage,
            change_sender: sender,
        }
    }

    /// 获取配置项
    pub async fn get(&self, key: &str) -> Result<Option<ConfigItem>, ConfigStoreError> {
        self.storage.get(key).await
    }

    /// 设置配置项
    pub async fn set(&self, mut item: ConfigItem, updated_by: String) -> Result<(), ConfigStoreError> {
        let mut old_item: Option<ConfigItem> = self.storage.get(&item.key).await?;
        
        if let Some(ref mut old) = old_item {
            old.update_value(item.value.clone(), updated_by.clone());
            item = old.clone();
        } else {
            item.updated_by = updated_by.clone();
        }

        self.storage.set(item.clone()).await?;

        // 发送变更事件
        let event = if old_item.is_some() {
            crate::ConfigChangeEvent::Updated(old_item.unwrap(), item)
        } else {
            crate::ConfigChangeEvent::Created(item)
        };

        let _ = self.change_sender.send(event);
        Ok(())
    }

    /// 删除配置项
    pub async fn delete(&self, key: &str) -> Result<(), ConfigStoreError> {
        self.storage.delete(key).await?;

        // 发送删除事件
        let event = crate::ConfigChangeEvent::Deleted(key.to_string());
        let _ = self.change_sender.send(event);
        Ok(())
    }

    /// 获取所有配置项
    pub async fn get_all(&self) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        self.storage.get_all().await
    }

    /// 按类别获取配置项
    pub async fn get_by_category(&self, category: &str) -> Result<Vec<ConfigItem>, ConfigStoreError> {
        self.storage.get_by_category(category).await
    }

    /// 批量设置配置项
    pub async fn set_batch(&self, items: Vec<ConfigItem>, updated_by: String) -> Result<(), ConfigStoreError> {
        let mut updated_items = Vec::new();
        
        for mut item in items {
            let mut old_item: Option<ConfigItem> = self.storage.get(&item.key).await?;
            
            if let Some(ref mut old) = old_item {
                old.update_value(item.value.clone(), updated_by.clone());
                updated_items.push(old.clone());
            } else {
                item.updated_by = updated_by.clone();
                updated_items.push(item);
            }
        }

        self.storage.set_batch(updated_items.clone()).await?;

        // 发送批量更新事件
        let event = crate::ConfigChangeEvent::BatchUpdated(updated_items);
        let _ = self.change_sender.send(event);
        Ok(())
    }

    /// 检查配置项是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, ConfigStoreError> {
        self.storage.exists(key).await
    }

    /// 获取变更事件接收器
    pub fn get_change_receiver(&self) -> tokio::sync::broadcast::Receiver<crate::ConfigChangeEvent> {
        self.change_sender.subscribe()
    }

    /// 创建内存存储
    pub fn new_memory() -> Self {
        let storage = Box::new(MemoryConfigStore::new());
        Self::new(storage)
    }

    /// 创建文件存储
    pub fn new_file(file_path: PathBuf) -> Self {
        let storage = Box::new(FileConfigStore::new(file_path));
        Self::new(storage)
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new_memory()
    }
}
