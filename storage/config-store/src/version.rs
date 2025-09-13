//! Configuration version management

use crate::ConfigStoreError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// 配置版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub id: Uuid,
    pub version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub changes: Vec<ConfigChange>,
    pub created_by: String,
    pub description: Option<String>,
    pub is_rollback: bool,
}

/// 配置变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub change_type: ConfigChangeType,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub old_version: Option<u64>,
    pub new_version: Option<u64>,
}

/// 配置变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigChangeType {
    Created,
    Updated,
    Deleted,
}

/// 版本管理器
pub struct VersionManager {
    versions: HashMap<u64, ConfigVersion>,
    current_version: u64,
    max_versions: usize,
}

impl VersionManager {
    pub fn new(max_versions: usize) -> Self {
        Self {
            versions: HashMap::new(),
            current_version: 0,
            max_versions,
        }
    }

    /// 创建新版本
    pub fn create_version(
        &mut self,
        changes: Vec<ConfigChange>,
        created_by: String,
        description: Option<String>,
    ) -> Result<ConfigVersion, ConfigStoreError> {
        self.current_version += 1;
        
        let version = ConfigVersion {
            id: Uuid::new_v4(),
            version: self.current_version,
            timestamp: chrono::Utc::now(),
            changes,
            created_by,
            description,
            is_rollback: false,
        };

        self.versions.insert(self.current_version, version.clone());
        
        // 清理旧版本
        self.cleanup_old_versions();
        
        info!("Created config version: {}", self.current_version);
        Ok(version)
    }

    /// 获取版本信息
    pub fn get_version(&self, version: u64) -> Option<&ConfigVersion> {
        self.versions.get(&version)
    }

    /// 获取当前版本
    pub fn get_current_version(&self) -> u64 {
        self.current_version
    }

    /// 获取所有版本
    pub fn get_all_versions(&self) -> Vec<&ConfigVersion> {
        self.versions.values().collect()
    }

    /// 获取版本历史
    pub fn get_version_history(&self, limit: Option<usize>) -> Vec<&ConfigVersion> {
        let mut versions: Vec<&ConfigVersion> = self.versions.values().collect();
        versions.sort_by(|a, b| b.version.cmp(&a.version));
        
        if let Some(limit) = limit {
            versions.truncate(limit);
        }
        
        versions
    }

    /// 回滚到指定版本
    pub fn rollback_to_version(
        &mut self,
        target_version: u64,
        created_by: String,
    ) -> Result<ConfigVersion, ConfigStoreError> {
        if target_version > self.current_version {
            return Err(ConfigStoreError::Validation(
                format!("Cannot rollback to future version: {}", target_version)
            ));
        }

        if target_version == self.current_version {
            return Err(ConfigStoreError::Validation(
                "Already at target version".to_string()
            ));
        }

        // 获取目标版本
        let _target_version_info = self.versions.get(&target_version)
            .ok_or_else(|| ConfigStoreError::NotFound(
                format!("Version {} not found", target_version)
            ))?;

        // 计算回滚变更
        let rollback_changes = self.calculate_rollback_changes(target_version)?;

        // 创建回滚版本
        self.current_version += 1;
        
        let rollback_version = ConfigVersion {
            id: Uuid::new_v4(),
            version: self.current_version,
            timestamp: chrono::Utc::now(),
            changes: rollback_changes,
            created_by,
            description: Some(format!("Rollback to version {}", target_version)),
            is_rollback: true,
        };

        self.versions.insert(self.current_version, rollback_version.clone());
        
        // 清理旧版本
        self.cleanup_old_versions();
        
        info!("Rolled back to version: {}", target_version);
        Ok(rollback_version)
    }

    /// 比较两个版本
    pub fn compare_versions(&self, version1: u64, version2: u64) -> Result<Vec<ConfigChange>, ConfigStoreError> {
        let _v1 = self.versions.get(&version1)
            .ok_or_else(|| ConfigStoreError::NotFound(format!("Version {} not found", version1)))?;
        
        let _v2 = self.versions.get(&version2)
            .ok_or_else(|| ConfigStoreError::NotFound(format!("Version {} not found", version2)))?;

        let mut changes = Vec::new();
        
        // 收集所有变更
        let mut all_changes = HashMap::new();
        
        // 从版本1到当前版本的所有变更
        for version in version1..=self.current_version {
            if let Some(version_info) = self.versions.get(&version) {
                for change in &version_info.changes {
                    all_changes.insert(change.key.clone(), change.clone());
                }
            }
        }
        
        // 从版本2到当前版本的所有变更
        let mut v2_changes = HashMap::new();
        for version in version2..=self.current_version {
            if let Some(version_info) = self.versions.get(&version) {
                for change in &version_info.changes {
                    v2_changes.insert(change.key.clone(), change.clone());
                }
            }
        }
        
        // 计算差异
        for (key, change1) in &all_changes {
            if let Some(change2) = v2_changes.get(key) {
                if change1.new_value != change2.new_value {
                    changes.push(ConfigChange {
                        key: key.clone(),
                        change_type: ConfigChangeType::Updated,
                        old_value: change1.new_value.clone(),
                        new_value: change2.new_value.clone(),
                        old_version: change1.new_version,
                        new_version: change2.new_version,
                    });
                }
            } else {
                changes.push(ConfigChange {
                    key: key.clone(),
                    change_type: ConfigChangeType::Deleted,
                    old_value: change1.new_value.clone(),
                    new_value: None,
                    old_version: change1.new_version,
                    new_version: None,
                });
            }
        }
        
        for (key, change2) in &v2_changes {
            if !all_changes.contains_key(key) {
                changes.push(ConfigChange {
                    key: key.clone(),
                    change_type: ConfigChangeType::Created,
                    old_value: None,
                    new_value: change2.new_value.clone(),
                    old_version: None,
                    new_version: change2.new_version,
                });
            }
        }
        
        Ok(changes)
    }

    /// 计算回滚变更
    fn calculate_rollback_changes(&self, target_version: u64) -> Result<Vec<ConfigChange>, ConfigStoreError> {
        let mut changes = Vec::new();
        
        // 从目标版本到当前版本的所有变更
        for version in target_version..=self.current_version {
            if let Some(version_info) = self.versions.get(&version) {
                for change in &version_info.changes {
                    // 创建反向变更
                    let rollback_change = ConfigChange {
                        key: change.key.clone(),
                        change_type: match change.change_type {
                            ConfigChangeType::Created => ConfigChangeType::Deleted,
                            ConfigChangeType::Updated => ConfigChangeType::Updated,
                            ConfigChangeType::Deleted => ConfigChangeType::Created,
                        },
                        old_value: change.new_value.clone(),
                        new_value: change.old_value.clone(),
                        old_version: change.new_version,
                        new_version: change.old_version,
                    };
                    changes.push(rollback_change);
                }
            }
        }
        
        Ok(changes)
    }

    /// 清理旧版本
    fn cleanup_old_versions(&mut self) {
        if self.versions.len() <= self.max_versions {
            return;
        }

        let mut versions_to_remove = Vec::new();
        let mut version_numbers: Vec<u64> = self.versions.keys().cloned().collect();
        version_numbers.sort();

        let remove_count = self.versions.len() - self.max_versions;
        for i in 0..remove_count {
            versions_to_remove.push(version_numbers[i]);
        }

        for version in versions_to_remove {
            self.versions.remove(&version);
            info!("Removed old config version: {}", version);
        }
    }

    /// 导出版本数据
    pub fn export_version(&self, version: u64) -> Result<String, ConfigStoreError> {
        let version_info = self.versions.get(&version)
            .ok_or_else(|| ConfigStoreError::NotFound(format!("Version {} not found", version)))?;
        
        serde_json::to_string_pretty(version_info)
            .map_err(|e| ConfigStoreError::Serialization(e))
    }

    /// 导入版本数据
    pub fn import_version(&mut self, data: &str) -> Result<(), ConfigStoreError> {
        let version: ConfigVersion = serde_json::from_str(data)
            .map_err(|e| ConfigStoreError::Serialization(e))?;
        
        if version.version > self.current_version {
            self.current_version = version.version;
        }
        
        self.versions.insert(version.version, version);
        Ok(())
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new(100) // 默认保留100个版本
    }
}
