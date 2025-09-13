//! Configuration file watcher for hot reloading

use crate::{ConfigChangeEvent, ConfigItem, ConfigStoreError};
use anyhow::Result;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::Duration;
use tracing::{info, warn, error, debug};

/// 配置监听器
pub struct ConfigWatcher {
    file_path: PathBuf,
    watcher: Option<RecommendedWatcher>,
    change_sender: broadcast::Sender<ConfigChangeEvent>,
    is_watching: Arc<RwLock<bool>>,
    debounce_duration: Duration,
}

impl ConfigWatcher {
    pub fn new(file_path: PathBuf) -> Self {
        let (sender, _receiver) = broadcast::channel(1000);
        Self {
            file_path,
            watcher: None,
            change_sender: sender,
            is_watching: Arc::new(RwLock::new(false)),
            debounce_duration: Duration::from_millis(500),
        }
    }

    /// 开始监听文件变化
    pub async fn start_watching(&mut self) -> Result<(), ConfigStoreError> {
        if *self.is_watching.read().await {
            return Ok(()); // 已经在监听
        }

        info!("Starting config file watcher for: {:?}", self.file_path);

        // 创建文件监听器
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.try_send(event);
            }
        }).map_err(|e| ConfigStoreError::Other(e.into()))?;

        // 开始监听文件
        watcher.watch(&self.file_path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigStoreError::Other(e.into()))?;

        self.watcher = Some(watcher);
        *self.is_watching.write().await = true;

        // 启动事件处理任务
        let file_path = self.file_path.clone();
        let change_sender = self.change_sender.clone();
        let debounce_duration = self.debounce_duration;
        let is_watching = Arc::clone(&self.is_watching);

        tokio::spawn(async move {
            let mut last_modified = std::time::Instant::now();
            
            while *is_watching.read().await {
                if let Some(event) = rx.recv().await {
                    debug!("File system event: {:?}", event);
                    
                    if Self::should_process_event(&event, &file_path) {
                        let now = std::time::Instant::now();
                        
                        // 防抖处理
                        if now.duration_since(last_modified) >= debounce_duration {
                            last_modified = now;
                            
                            if let Err(e) = Self::process_file_change(&file_path, &change_sender).await {
                                error!("Failed to process file change: {}", e);
                            }
                        }
                    }
                }
            }
        });

        info!("Config file watcher started successfully");
        Ok(())
    }

    /// 停止监听文件变化
    pub async fn stop_watching(&mut self) -> Result<(), ConfigStoreError> {
        if !*self.is_watching.read().await {
            return Ok(()); // 已经停止监听
        }

        info!("Stopping config file watcher");

        *self.is_watching.write().await = false;
        self.watcher = None;

        info!("Config file watcher stopped");
        Ok(())
    }

    /// 获取变更事件接收器
    pub fn get_change_receiver(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.change_sender.subscribe()
    }

    /// 检查是否应该处理事件
    fn should_process_event(event: &Event, file_path: &PathBuf) -> bool {
        // 检查事件是否与我们的文件相关
        for path in &event.paths {
            if path == file_path {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => return true,
                    _ => return false,
                }
            }
        }
        false
    }

    /// 处理文件变化
    async fn process_file_change(
        file_path: &PathBuf,
        change_sender: &broadcast::Sender<ConfigChangeEvent>,
    ) -> Result<(), ConfigStoreError> {
        if !file_path.exists() {
            warn!("Config file does not exist: {:?}", file_path);
            return Ok(());
        }

        info!("Processing config file change: {:?}", file_path);

        // 读取文件内容
        let content = tokio::fs::read_to_string(file_path).await?;
        
        // 解析配置
        let configs: std::collections::HashMap<String, ConfigItem> = 
            serde_json::from_str(&content)
                .map_err(|e| ConfigStoreError::Serialization(e))?;

        // 发送批量更新事件
        let items: Vec<ConfigItem> = configs.into_values().collect();
        let event = ConfigChangeEvent::BatchUpdated(items);
        
        if let Err(e) = change_sender.send(event) {
            error!("Failed to send config change event: {}", e);
        }

        info!("Config file change processed successfully");
        Ok(())
    }

    /// 设置防抖时间
    pub fn set_debounce_duration(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    /// 手动触发文件重新加载
    pub async fn reload_file(&self) -> Result<(), ConfigStoreError> {
        Self::process_file_change(&self.file_path, &self.change_sender).await
    }
}

/// 配置热重载管理器
pub struct ConfigHotReloadManager {
    watchers: std::collections::HashMap<String, ConfigWatcher>,
    global_change_sender: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigHotReloadManager {
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(1000);
        Self {
            watchers: std::collections::HashMap::new(),
            global_change_sender: sender,
        }
    }

    /// 添加配置文件监听
    pub async fn add_watcher(&mut self, name: String, file_path: PathBuf) -> Result<(), ConfigStoreError> {
        let mut watcher = ConfigWatcher::new(file_path);
        watcher.start_watching().await?;
        
        // 转发事件到全局发送器
        let global_sender = self.global_change_sender.clone();
        let mut receiver = watcher.get_change_receiver();
        
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let Err(e) = global_sender.send(event) {
                    error!("Failed to forward config change event: {}", e);
                }
            }
        });
        
        self.watchers.insert(name.clone(), watcher);
        info!("Added config watcher: {}", name);
        Ok(())
    }

    /// 移除配置文件监听
    pub async fn remove_watcher(&mut self, name: &str) -> Result<(), ConfigStoreError> {
        if let Some(mut watcher) = self.watchers.remove(name) {
            watcher.stop_watching().await?;
            info!("Removed config watcher: {}", name);
        }
        Ok(())
    }

    /// 获取全局变更事件接收器
    pub fn get_global_change_receiver(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.global_change_sender.subscribe()
    }

    /// 获取所有监听器状态
    pub fn get_watcher_status(&self) -> std::collections::HashMap<String, bool> {
        self.watchers.iter()
            .map(|(name, _)| (name.clone(), true)) // 简化实现
            .collect()
    }

    /// 停止所有监听器
    pub async fn stop_all_watchers(&mut self) -> Result<(), ConfigStoreError> {
        for (name, mut watcher) in self.watchers.drain() {
            if let Err(e) = watcher.stop_watching().await {
                error!("Failed to stop watcher {}: {}", name, e);
            }
        }
        info!("Stopped all config watchers");
        Ok(())
    }
}

impl Default for ConfigHotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConfigHotReloadManager {
    fn drop(&mut self) {
        // 在drop时停止所有监听器
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            handle.block_on(async {
                for (_, mut watcher) in self.watchers.drain() {
                    let _ = watcher.stop_watching().await;
                }
            });
        }
    }
}
