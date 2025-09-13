//! Event indexing system

use crate::{Event, EventStoreError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use tracing::info;
use uuid::Uuid;

/// 索引类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexType {
    /// 哈希索引
    Hash,
    /// B树索引
    BTree,
    /// 位图索引
    Bitmap,
}

/// 索引字段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IndexField {
    /// 事件类型
    EventType,
    /// 会话ID
    SessionId,
    /// 用户ID
    UserId,
    /// 来源
    Source,
    /// 时间戳
    Timestamp,
    /// 严重级别
    Severity,
}

/// 索引定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub field: IndexField,
    pub index_type: IndexType,
    pub unique: bool,
}

/// 事件索引
pub struct EventIndex {
    definition: IndexDefinition,
    hash_index: Option<HashMap<String, Vec<Uuid>>>,
    btree_index: Option<BTreeMap<String, Vec<Uuid>>>,
    bitmap_index: Option<HashMap<String, Vec<bool>>>,
}

impl EventIndex {
    pub fn new(definition: IndexDefinition) -> Self {
        let hash_index = if definition.index_type == IndexType::Hash {
            Some(HashMap::new())
        } else {
            None
        };

        let btree_index = if definition.index_type == IndexType::BTree {
            Some(BTreeMap::new())
        } else {
            None
        };

        let bitmap_index = if definition.index_type == IndexType::Bitmap {
            Some(HashMap::new())
        } else {
            None
        };

        Self {
            definition,
            hash_index,
            btree_index,
            bitmap_index,
        }
    }

    /// 添加事件到索引
    pub fn add_event(&mut self, event: &Event) -> Result<(), EventStoreError> {
        let key = self.extract_key(event)?;
        
        match self.definition.index_type {
            IndexType::Hash => {
                if let Some(ref mut index) = self.hash_index {
                    index.entry(key).or_insert_with(Vec::new).push(event.id);
                }
            }
            IndexType::BTree => {
                if let Some(ref mut index) = self.btree_index {
                    index.entry(key).or_insert_with(Vec::new).push(event.id);
                }
            }
            IndexType::Bitmap => {
                // 位图索引的简化实现
                if let Some(ref mut index) = self.bitmap_index {
                    index.entry(key).or_insert_with(Vec::new).push(true);
                }
            }
        }

        Ok(())
    }

    /// 从索引中移除事件
    pub fn remove_event(&mut self, event: &Event) -> Result<(), EventStoreError> {
        let key = self.extract_key(event)?;
        
        match self.definition.index_type {
            IndexType::Hash => {
                if let Some(ref mut index) = self.hash_index {
                    if let Some(event_ids) = index.get_mut(&key) {
                        event_ids.retain(|&id| id != event.id);
                        if event_ids.is_empty() {
                            index.remove(&key);
                        }
                    }
                }
            }
            IndexType::BTree => {
                if let Some(ref mut index) = self.btree_index {
                    if let Some(event_ids) = index.get_mut(&key) {
                        event_ids.retain(|&id| id != event.id);
                        if event_ids.is_empty() {
                            index.remove(&key);
                        }
                    }
                }
            }
            IndexType::Bitmap => {
                if let Some(ref mut index) = self.bitmap_index {
                    if let Some(bitmap) = index.get_mut(&key) {
                        bitmap.pop();
                        if bitmap.is_empty() {
                            index.remove(&key);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 查找事件ID
    pub fn find_events(&self, key: &str) -> Vec<Uuid> {
        match self.definition.index_type {
            IndexType::Hash => {
                if let Some(ref index) = self.hash_index {
                    index.get(key).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            IndexType::BTree => {
                if let Some(ref index) = self.btree_index {
                    index.get(key).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            IndexType::Bitmap => {
                if let Some(ref index) = self.bitmap_index {
                    if index.contains_key(key) {
                        // 位图索引的简化实现，返回空向量
                        Vec::new()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// 获取所有键
    pub fn get_all_keys(&self) -> Vec<String> {
        match self.definition.index_type {
            IndexType::Hash => {
                if let Some(ref index) = self.hash_index {
                    index.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            IndexType::BTree => {
                if let Some(ref index) = self.btree_index {
                    index.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            IndexType::Bitmap => {
                if let Some(ref index) = self.bitmap_index {
                    index.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// 获取索引统计信息
    pub fn get_stats(&self) -> IndexStats {
        let key_count = self.get_all_keys().len();
        let total_entries = match self.definition.index_type {
            IndexType::Hash => {
                if let Some(ref index) = self.hash_index {
                    index.values().map(|v| v.len()).sum()
                } else {
                    0
                }
            }
            IndexType::BTree => {
                if let Some(ref index) = self.btree_index {
                    index.values().map(|v| v.len()).sum()
                } else {
                    0
                }
            }
            IndexType::Bitmap => {
                if let Some(ref index) = self.bitmap_index {
                    index.values().map(|v| v.len()).sum()
                } else {
                    0
                }
            }
        };

        IndexStats {
            name: self.definition.name.clone(),
            field: format!("{:?}", self.definition.field),
            index_type: format!("{:?}", self.definition.index_type),
            key_count,
            total_entries,
            unique: self.definition.unique,
        }
    }

    /// 提取索引键
    fn extract_key(&self, event: &Event) -> Result<String, EventStoreError> {
        let key = match self.definition.field {
            IndexField::EventType => format!("{:?}", event.event_type),
            IndexField::SessionId => event.session_id.clone().unwrap_or_default(),
            IndexField::UserId => event.user_id.map(|id| id.to_string()).unwrap_or_default(),
            IndexField::Source => event.source.clone(),
            IndexField::Timestamp => event.timestamp.to_rfc3339(),
            IndexField::Severity => format!("{:?}", event.severity),
        };

        Ok(key)
    }
}

/// 索引统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub name: String,
    pub field: String,
    pub index_type: String,
    pub key_count: usize,
    pub total_entries: usize,
    pub unique: bool,
}

/// 索引管理器
pub struct IndexManager {
    indexes: HashMap<String, EventIndex>,
}

impl IndexManager {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }

    /// 创建索引
    pub fn create_index(&mut self, definition: IndexDefinition) -> Result<(), EventStoreError> {
        if self.indexes.contains_key(&definition.name) {
            return Err(EventStoreError::Storage(format!("Index '{}' already exists", definition.name)));
        }

        let index = EventIndex::new(definition.clone());
        self.indexes.insert(definition.name.clone(), index);
        
        info!("Created index: {} on field {:?}", definition.name, definition.field);
        Ok(())
    }

    /// 删除索引
    pub fn drop_index(&mut self, name: &str) -> Result<(), EventStoreError> {
        if self.indexes.remove(name).is_some() {
            info!("Dropped index: {}", name);
            Ok(())
        } else {
            Err(EventStoreError::Storage(format!("Index '{}' not found", name)))
        }
    }

    /// 获取索引
    pub fn get_index(&self, name: &str) -> Option<&EventIndex> {
        self.indexes.get(name)
    }

    /// 获取索引（可变）
    pub fn get_index_mut(&mut self, name: &str) -> Option<&mut EventIndex> {
        self.indexes.get_mut(name)
    }

    /// 添加事件到所有索引
    pub fn add_event_to_all_indexes(&mut self, event: &Event) -> Result<(), EventStoreError> {
        for index in self.indexes.values_mut() {
            index.add_event(event)?;
        }
        Ok(())
    }

    /// 从所有索引中移除事件
    pub fn remove_event_from_all_indexes(&mut self, event: &Event) -> Result<(), EventStoreError> {
        for index in self.indexes.values_mut() {
            index.remove_event(event)?;
        }
        Ok(())
    }

    /// 根据索引查找事件
    pub fn find_events_by_index(&self, index_name: &str, key: &str) -> Result<Vec<Uuid>, EventStoreError> {
        if let Some(index) = self.indexes.get(index_name) {
            Ok(index.find_events(key))
        } else {
            Err(EventStoreError::Storage(format!("Index '{}' not found", index_name)))
        }
    }

    /// 获取所有索引统计信息
    pub fn get_all_index_stats(&self) -> Vec<IndexStats> {
        self.indexes.values().map(|index| index.get_stats()).collect()
    }

    /// 获取索引列表
    pub fn get_index_names(&self) -> Vec<String> {
        self.indexes.keys().cloned().collect()
    }

    /// 重建所有索引
    pub fn rebuild_all_indexes(&mut self, events: &[Event]) -> Result<(), EventStoreError> {
        info!("Rebuilding all indexes with {} events", events.len());
        
        // 清空所有索引
        for index in self.indexes.values_mut() {
            match index.definition.index_type {
                IndexType::Hash => {
                    if let Some(ref mut hash_index) = index.hash_index {
                        hash_index.clear();
                    }
                }
                IndexType::BTree => {
                    if let Some(ref mut btree_index) = index.btree_index {
                        btree_index.clear();
                    }
                }
                IndexType::Bitmap => {
                    if let Some(ref mut bitmap_index) = index.bitmap_index {
                        bitmap_index.clear();
                    }
                }
            }
        }

        // 重新添加所有事件
        for event in events {
            self.add_event_to_all_indexes(event)?;
        }

        info!("Rebuilt all indexes successfully");
        Ok(())
    }

    /// 创建默认索引
    pub fn create_default_indexes(&mut self) -> Result<(), EventStoreError> {
        let default_indexes = vec![
            IndexDefinition {
                name: "event_type_idx".to_string(),
                field: IndexField::EventType,
                index_type: IndexType::Hash,
                unique: false,
            },
            IndexDefinition {
                name: "session_id_idx".to_string(),
                field: IndexField::SessionId,
                index_type: IndexType::Hash,
                unique: false,
            },
            IndexDefinition {
                name: "user_id_idx".to_string(),
                field: IndexField::UserId,
                index_type: IndexType::Hash,
                unique: false,
            },
            IndexDefinition {
                name: "timestamp_idx".to_string(),
                field: IndexField::Timestamp,
                index_type: IndexType::BTree,
                unique: false,
            },
            IndexDefinition {
                name: "severity_idx".to_string(),
                field: IndexField::Severity,
                index_type: IndexType::Hash,
                unique: false,
            },
        ];

        for definition in default_indexes {
            self.create_index(definition)?;
        }

        info!("Created default indexes");
        Ok(())
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}
