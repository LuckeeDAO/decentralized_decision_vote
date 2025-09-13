//! Configuration caching system

use crate::{ConfigItem, ConfigChangeEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, debug};

/// 缓存策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheStrategy {
    /// 不缓存
    NoCache,
    /// 固定TTL缓存
    FixedTtl(Duration),
    /// LRU缓存
    Lru(usize),
    /// 写入时失效
    WriteThrough,
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    item: ConfigItem,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

impl CacheEntry {
    fn new(item: ConfigItem) -> Self {
        let now = Instant::now();
        Self {
            item,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// 配置缓存
pub struct ConfigCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    strategy: CacheStrategy,
    max_size: usize,
    change_receiver: broadcast::Receiver<ConfigChangeEvent>,
}

impl ConfigCache {
    pub fn new(strategy: CacheStrategy, max_size: usize) -> Self {
        let (_, receiver) = broadcast::channel(1000);
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            max_size,
            change_receiver: receiver,
        }
    }

    /// 设置变更事件接收器
    pub fn set_change_receiver(&mut self, receiver: broadcast::Receiver<ConfigChangeEvent>) {
        self.change_receiver = receiver;
    }

    /// 获取配置项
    pub async fn get(&self, key: &str) -> Option<ConfigItem> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(key) {
            // 检查是否过期
            if let CacheStrategy::FixedTtl(ttl) = self.strategy {
                if entry.is_expired(ttl) {
                    cache.remove(key);
                    debug!("Cache entry expired for key: {}", key);
                    return None;
                }
            }
            
            entry.touch();
            debug!("Cache hit for key: {}", key);
            return Some(entry.item.clone());
        }
        
        debug!("Cache miss for key: {}", key);
        None
    }

    /// 设置配置项
    pub async fn set(&self, item: ConfigItem) {
        let mut cache = self.cache.write().await;
        
        // 检查缓存大小限制
        if cache.len() >= self.max_size {
            self.evict_entries(&mut cache).await;
        }
        
        let entry = CacheEntry::new(item);
        let key = entry.item.key.clone();
        cache.insert(key.clone(), entry);
        debug!("Cached item: {}", key);
    }

    /// 删除配置项
    pub async fn remove(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        debug!("Removed from cache: {}", key);
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cache cleared");
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let mut total_access_count = 0;
        let mut total_age = Duration::ZERO;
        
        for entry in cache.values() {
            total_access_count += entry.access_count;
            total_age += entry.created_at.elapsed();
        }
        
        let avg_access_count = if cache.is_empty() {
            0.0
        } else {
            total_access_count as f64 / cache.len() as f64
        };
        
        let avg_age = if cache.is_empty() {
            Duration::ZERO
        } else {
            total_age / cache.len() as u32
        };
        
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hit_rate: 0.0, // 需要跟踪命中率
            total_access_count,
            avg_access_count,
            avg_age,
            strategy: self.strategy.clone(),
        }
    }

    /// 启动缓存清理任务
    pub async fn start_cleanup_task(&self) {
        let cache = Arc::clone(&self.cache);
        let strategy = self.strategy.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                if let CacheStrategy::FixedTtl(ttl) = strategy {
                    let mut cache_guard = cache.write().await;
                    let expired_keys: Vec<String> = cache_guard
                        .iter()
                        .filter(|(_, entry)| entry.is_expired(ttl))
                        .map(|(key, _)| key.clone())
                        .collect();
                    
                    for key in expired_keys {
                        cache_guard.remove(&key);
                        debug!("Removed expired cache entry: {}", key);
                    }
                }
            }
        });
    }

    /// 启动变更事件处理任务
    pub async fn start_change_handler(&mut self) {
        let cache = Arc::clone(&self.cache);
        let mut change_receiver = self.change_receiver.resubscribe();
        
        tokio::spawn(async move {
            while let Ok(event) = change_receiver.recv().await {
                match event {
                    ConfigChangeEvent::Created(item) => {
                        let mut cache_guard = cache.write().await;
                        let entry = CacheEntry::new(item);
                        cache_guard.insert(entry.item.key.clone(), entry);
                    }
                    ConfigChangeEvent::Updated(_, new_item) => {
                        let mut cache_guard = cache.write().await;
                        let entry = CacheEntry::new(new_item);
                        cache_guard.insert(entry.item.key.clone(), entry);
                    }
                    ConfigChangeEvent::Deleted(key) => {
                        let mut cache_guard = cache.write().await;
                        cache_guard.remove(&key);
                    }
                    ConfigChangeEvent::BatchUpdated(items) => {
                        let mut cache_guard = cache.write().await;
                        for item in items {
                            let entry = CacheEntry::new(item);
                            cache_guard.insert(entry.item.key.clone(), entry);
                        }
                    }
                }
            }
        });
    }

    /// 驱逐缓存条目
    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        match self.strategy {
            CacheStrategy::Lru(max_size) => {
                if cache.len() >= max_size {
                    // 找到最少使用的条目
                    let mut entries: Vec<(String, Instant, u64)> = cache
                        .iter()
                        .map(|(key, entry)| (key.clone(), entry.last_accessed, entry.access_count))
                        .collect();
                    
                    entries.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
                    
                    // 移除最旧的条目
                    let to_remove = entries.len() - max_size + 1;
                    for (key, _, _) in entries.iter().take(to_remove) {
                        cache.remove(key);
                        debug!("Evicted cache entry: {}", key);
                    }
                }
            }
            _ => {
                // 其他策略的驱逐逻辑
                if cache.len() >= self.max_size {
                    let keys_to_remove: Vec<String> = cache.keys().take(10).cloned().collect();
                    for key in keys_to_remove {
                        cache.remove(&key);
                        debug!("Evicted cache entry: {}", key);
                    }
                }
            }
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_rate: f64,
    pub total_access_count: u64,
    pub avg_access_count: f64,
    pub avg_age: Duration,
    pub strategy: CacheStrategy,
}

/// 缓存管理器
pub struct CacheManager {
    caches: HashMap<String, ConfigCache>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }

    /// 创建缓存
    pub fn create_cache(&mut self, name: String, strategy: CacheStrategy, max_size: usize) -> &mut ConfigCache {
        let cache = ConfigCache::new(strategy, max_size);
        self.caches.insert(name.clone(), cache);
        self.caches.get_mut(&name).unwrap()
    }

    /// 获取缓存
    pub fn get_cache(&self, name: &str) -> Option<&ConfigCache> {
        self.caches.get(name)
    }

    /// 获取缓存（可变）
    pub fn get_cache_mut(&mut self, name: &str) -> Option<&mut ConfigCache> {
        self.caches.get_mut(name)
    }

    /// 删除缓存
    pub fn remove_cache(&mut self, name: &str) {
        self.caches.remove(name);
    }

    /// 获取所有缓存统计信息
    pub async fn get_all_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        
        for (name, cache) in &self.caches {
            stats.insert(name.clone(), cache.get_stats().await);
        }
        
        stats
    }

    /// 清空所有缓存
    pub async fn clear_all_caches(&self) {
        for cache in self.caches.values() {
            cache.clear().await;
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}
