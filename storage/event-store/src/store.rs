//! Event storage implementations

use crate::{EventStorage, Event, EventType};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use uuid::Uuid;

/// 事件存储错误
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Event not found: {0}")]
    NotFound(Uuid),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// 内存事件存储
pub struct MemoryEventStore {
    events: Arc<RwLock<HashMap<Uuid, Event>>>,
    session_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    user_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    type_index: Arc<RwLock<HashMap<EventType, Vec<Uuid>>>>,
}

impl MemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            session_index: Arc::new(RwLock::new(HashMap::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn update_indexes(&self, event: &Event) {
        // 更新会话索引
        if let Some(session_id) = &event.session_id {
            let mut session_index = self.session_index.write().await;
            session_index.entry(session_id.clone()).or_insert_with(Vec::new).push(event.id);
        }

        // 更新用户索引
        if let Some(user_id) = event.user_id {
            let mut user_index = self.user_index.write().await;
            user_index.entry(user_id).or_insert_with(Vec::new).push(event.id);
        }

        // 更新类型索引
        let mut type_index = self.type_index.write().await;
        type_index.entry(event.event_type.clone()).or_insert_with(Vec::new).push(event.id);
    }
}

#[async_trait]
impl EventStorage for MemoryEventStore {
    async fn store_event(&self, event: Event) -> Result<(), EventStoreError> {
        let event_id = event.id;
        
        // 存储事件
        {
            let mut events = self.events.write().await;
            events.insert(event_id, event.clone());
        }
        
        // 更新索引
        self.update_indexes(&event).await;
        
        info!("Stored event: {} of type {:?}", event_id, event.event_type);
        Ok(())
    }

    async fn store_events(&self, events: Vec<Event>) -> Result<(), EventStoreError> {
        for event in events {
            self.store_event(event).await?;
        }
        Ok(())
    }

    async fn get_event(&self, event_id: Uuid) -> Result<Option<Event>, EventStoreError> {
        let events = self.events.read().await;
        Ok(events.get(&event_id).cloned())
    }

    async fn get_events_by_session(&self, session_id: &str) -> Result<Vec<Event>, EventStoreError> {
        let session_index = self.session_index.read().await;
        let event_ids = session_index.get(session_id).cloned().unwrap_or_default();
        
        let events = self.events.read().await;
        let mut result = Vec::new();
        
        for event_id in event_ids {
            if let Some(event) = events.get(&event_id) {
                result.push(event.clone());
            }
        }
        
        // 按时间排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn get_events_by_user(&self, user_id: Uuid) -> Result<Vec<Event>, EventStoreError> {
        let user_index = self.user_index.read().await;
        let event_ids = user_index.get(&user_id).cloned().unwrap_or_default();
        
        let events = self.events.read().await;
        let mut result = Vec::new();
        
        for event_id in event_ids {
            if let Some(event) = events.get(&event_id) {
                result.push(event.clone());
            }
        }
        
        // 按时间排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn get_events_by_time_range(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventStoreError> {
        let events = self.events.read().await;
        let mut result: Vec<Event> = events
            .values()
            .filter(|event| event.timestamp >= start_time && event.timestamp <= end_time)
            .cloned()
            .collect();
        
        // 按时间排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn get_events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError> {
        let type_index = self.type_index.read().await;
        let event_ids = type_index.get(event_type).cloned().unwrap_or_default();
        
        let events = self.events.read().await;
        let mut result = Vec::new();
        
        for event_id in event_ids {
            if let Some(event) = events.get(&event_id) {
                result.push(event.clone());
            }
        }
        
        // 按时间排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn get_all_events(&self) -> Result<Vec<Event>, EventStoreError> {
        let events = self.events.read().await;
        let mut result: Vec<Event> = events.values().cloned().collect();
        
        // 按时间排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn delete_event(&self, event_id: Uuid) -> Result<(), EventStoreError> {
        let event = {
            let mut events = self.events.write().await;
            events.remove(&event_id)
        };
        
        if let Some(event) = event {
            // 从索引中移除
            if let Some(session_id) = &event.session_id {
                let mut session_index = self.session_index.write().await;
                if let Some(event_ids) = session_index.get_mut(session_id) {
                    event_ids.retain(|&id| id != event_id);
                }
            }
            
            if let Some(user_id) = event.user_id {
                let mut user_index = self.user_index.write().await;
                if let Some(event_ids) = user_index.get_mut(&user_id) {
                    event_ids.retain(|&id| id != event_id);
                }
            }
            
            let mut type_index = self.type_index.write().await;
            if let Some(event_ids) = type_index.get_mut(&event.event_type) {
                event_ids.retain(|&id| id != event_id);
            }
            
            info!("Deleted event: {}", event_id);
        }
        
        Ok(())
    }

    async fn cleanup_expired_events(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64, EventStoreError> {
        let expired_events: Vec<Uuid> = {
            let events = self.events.read().await;
            events
                .iter()
                .filter(|(_, event)| event.timestamp < before)
                .map(|(id, _)| *id)
                .collect()
        };
        
        let count = expired_events.len() as u64;
        
        for event_id in expired_events {
            self.delete_event(event_id).await?;
        }
        
        info!("Cleaned up {} expired events", count);
        Ok(count)
    }
}

/// 文件事件存储
pub struct FileEventStore {
    file_path: PathBuf,
    memory_store: MemoryEventStore,
}

impl FileEventStore {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            memory_store: MemoryEventStore::new(),
        }
    }

    /// 从文件加载事件
    pub async fn load_from_file(&self) -> Result<(), EventStoreError> {
        if !self.file_path.exists() {
            info!("Event file does not exist, creating empty store");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let events: Vec<Event> = serde_json::from_str(&content)?;

        for event in events {
            self.memory_store.store_event(event).await?;
        }
        
        info!("Loaded {} events from file", self.memory_store.events.read().await.len());
        Ok(())
    }

    /// 保存事件到文件
    pub async fn save_to_file(&self) -> Result<(), EventStoreError> {
        let events = self.memory_store.get_all_events().await?;
        let content = serde_json::to_string_pretty(&events)?;
        
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&self.file_path, content).await?;
        info!("Saved {} events to file", events.len());
        Ok(())
    }
}

#[async_trait]
impl EventStorage for FileEventStore {
    async fn store_event(&self, event: Event) -> Result<(), EventStoreError> {
        self.memory_store.store_event(event).await?;
        self.save_to_file().await?;
        Ok(())
    }

    async fn store_events(&self, events: Vec<Event>) -> Result<(), EventStoreError> {
        self.memory_store.store_events(events).await?;
        self.save_to_file().await?;
        Ok(())
    }

    async fn get_event(&self, event_id: Uuid) -> Result<Option<Event>, EventStoreError> {
        self.memory_store.get_event(event_id).await
    }

    async fn get_events_by_session(&self, session_id: &str) -> Result<Vec<Event>, EventStoreError> {
        self.memory_store.get_events_by_session(session_id).await
    }

    async fn get_events_by_user(&self, user_id: Uuid) -> Result<Vec<Event>, EventStoreError> {
        self.memory_store.get_events_by_user(user_id).await
    }

    async fn get_events_by_time_range(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventStoreError> {
        self.memory_store.get_events_by_time_range(start_time, end_time).await
    }

    async fn get_events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError> {
        self.memory_store.get_events_by_type(event_type).await
    }

    async fn get_all_events(&self) -> Result<Vec<Event>, EventStoreError> {
        self.memory_store.get_all_events().await
    }

    async fn delete_event(&self, event_id: Uuid) -> Result<(), EventStoreError> {
        self.memory_store.delete_event(event_id).await?;
        self.save_to_file().await?;
        Ok(())
    }

    async fn cleanup_expired_events(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64, EventStoreError> {
        let count = self.memory_store.cleanup_expired_events(before).await?;
        self.save_to_file().await?;
        Ok(count)
    }
}

/// 事件存储管理器
pub struct EventStore {
    storage: Box<dyn EventStorage>,
}

impl EventStore {
    pub fn new(storage: Box<dyn EventStorage>) -> Self {
        Self { storage }
    }

    /// 存储事件
    pub async fn store_event(&self, event: Event) -> Result<(), EventStoreError> {
        self.storage.store_event(event).await
    }

    /// 批量存储事件
    pub async fn store_events(&self, events: Vec<Event>) -> Result<(), EventStoreError> {
        self.storage.store_events(events).await
    }

    /// 根据ID获取事件
    pub async fn get_event(&self, event_id: Uuid) -> Result<Option<Event>, EventStoreError> {
        self.storage.get_event(event_id).await
    }

    /// 根据会话ID获取事件
    pub async fn get_events_by_session(&self, session_id: &str) -> Result<Vec<Event>, EventStoreError> {
        self.storage.get_events_by_session(session_id).await
    }

    /// 根据用户ID获取事件
    pub async fn get_events_by_user(&self, user_id: Uuid) -> Result<Vec<Event>, EventStoreError> {
        self.storage.get_events_by_user(user_id).await
    }

    /// 根据时间范围获取事件
    pub async fn get_events_by_time_range(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventStoreError> {
        self.storage.get_events_by_time_range(start_time, end_time).await
    }

    /// 根据事件类型获取事件
    pub async fn get_events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError> {
        self.storage.get_events_by_type(event_type).await
    }

    /// 获取所有事件
    pub async fn get_all_events(&self) -> Result<Vec<Event>, EventStoreError> {
        self.storage.get_all_events().await
    }

    /// 删除事件
    pub async fn delete_event(&self, event_id: Uuid) -> Result<(), EventStoreError> {
        self.storage.delete_event(event_id).await
    }

    /// 清理过期事件
    pub async fn cleanup_expired_events(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64, EventStoreError> {
        self.storage.cleanup_expired_events(before).await
    }

    /// 创建内存存储
    pub fn new_memory() -> Self {
        let storage = Box::new(MemoryEventStore::new());
        Self::new(storage)
    }

    /// 创建文件存储
    pub fn new_file(file_path: PathBuf) -> Self {
        let storage = Box::new(FileEventStore::new(file_path));
        Self::new(storage)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new_memory()
    }
}
