//! 区块链存储与现有存储系统集成示例

use blockchain_store::{
    BlockchainManager, BlockchainConfig, BlockchainType, 
    StorageTransaction, Result
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 集成存储系统
pub struct IntegratedStorage {
    blockchain_manager: BlockchainManager,
    local_cache: HashMap<String, Vec<u8>>,
}

/// 存储策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageStrategy {
    /// 仅本地存储
    LocalOnly,
    /// 仅区块链存储
    BlockchainOnly,
    /// 本地 + 区块链双重存储
    Dual,
    /// 根据数据大小自动选择
    Auto,
}

/// 存储元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub key: String,
    pub size: u64,
    pub strategy: StorageStrategy,
    pub blockchain_tx: Option<StorageTransaction>,
    pub local_timestamp: chrono::DateTime<chrono::Utc>,
    pub blockchain_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl IntegratedStorage {
    /// 创建新的集成存储系统
    pub fn new(blockchain_config: BlockchainConfig) -> Self {
        Self {
            blockchain_manager: BlockchainManager::new(blockchain_config),
            local_cache: HashMap::new(),
        }
    }

    /// 初始化存储系统
    pub async fn initialize(&mut self) -> Result<()> {
        self.blockchain_manager.initialize().await?;
        Ok(())
    }

    /// 存储数据（智能选择存储策略）
    pub async fn store_data(
        &mut self,
        key: &str,
        data: &[u8],
        strategy: StorageStrategy,
        metadata: Option<serde_json::Value>,
    ) -> Result<StorageMetadata> {
        let now = chrono::Utc::now();
        let mut storage_metadata = StorageMetadata {
            key: key.to_string(),
            size: data.len() as u64,
            strategy: strategy.clone(),
            blockchain_tx: None,
            local_timestamp: now,
            blockchain_timestamp: None,
        };

        match strategy {
            StorageStrategy::LocalOnly => {
                self.store_local(key, data).await?;
            }
            StorageStrategy::BlockchainOnly => {
                let tx = self.store_blockchain(key, data, metadata).await?;
                storage_metadata.blockchain_tx = Some(tx.clone());
                storage_metadata.blockchain_timestamp = Some(tx.timestamp);
            }
            StorageStrategy::Dual => {
                // 本地存储
                self.store_local(key, data).await?;
                
                // 区块链存储
                let tx = self.store_blockchain(key, data, metadata).await?;
                storage_metadata.blockchain_tx = Some(tx.clone());
                storage_metadata.blockchain_timestamp = Some(tx.timestamp);
            }
            StorageStrategy::Auto => {
                // 根据数据大小自动选择策略
                if data.len() < 1024 { // 小于 1KB 使用本地存储
                    self.store_local(key, data).await?;
                } else { // 大于 1KB 使用区块链存储
                    let tx = self.store_blockchain(key, data, metadata).await?;
                    storage_metadata.blockchain_tx = Some(tx.clone());
                    storage_metadata.blockchain_timestamp = Some(tx.timestamp);
                }
            }
        }

        Ok(storage_metadata)
    }

    /// 检索数据
    pub async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>> {
        // 首先尝试从本地缓存检索
        if let Some(data) = self.local_cache.get(key) {
            return Ok(data.clone());
        }

        // 如果本地没有，尝试从区块链检索
        self.retrieve_from_blockchain(key).await
    }

    /// 验证数据完整性
    pub async fn verify_data(&self, key: &str, expected_hash: &str) -> Result<bool> {
        let data = self.retrieve_data(key).await?;
        let actual_hash = hex::encode(sha2::Sha256::digest(&data));
        Ok(actual_hash == expected_hash)
    }

    /// 获取存储统计
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let blockchain_stats = self.blockchain_manager.get_all_stats().await?;
        
        Ok(StorageStats {
            local_items: self.local_cache.len() as u64,
            local_size: self.local_cache.values().map(|v| v.len() as u64).sum(),
            blockchain_stats,
            total_items: self.local_cache.len() as u64 + blockchain_stats.values().map(|s| s.total_transactions).sum::<u64>(),
        })
    }

    /// 本地存储
    async fn store_local(&mut self, key: &str, data: &[u8]) -> Result<()> {
        self.local_cache.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    /// 区块链存储
    async fn store_blockchain(
        &self,
        key: &str,
        data: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction> {
        self.blockchain_manager.store_data(
            &BlockchainType::Ethereum, // 使用默认区块链
            key,
            data,
            metadata,
        ).await
    }

    /// 从区块链检索
    async fn retrieve_from_blockchain(&self, key: &str) -> Result<Vec<u8>> {
        self.blockchain_manager.retrieve_data(
            &BlockchainType::Ethereum,
            key,
        ).await
    }
}

/// 存储统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub local_items: u64,
    pub local_size: u64,
    pub blockchain_stats: HashMap<BlockchainType, blockchain_store::StorageStats>,
    pub total_items: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载区块链配置
    let blockchain_config = BlockchainConfig::from_file("examples/config.json")
        .map_err(|e| blockchain_store::BlockchainError::InvalidConfig(e.to_string()))?;

    // 创建集成存储系统
    let mut storage = IntegratedStorage::new(blockchain_config);
    storage.initialize().await?;

    println!("=== 集成存储系统示例 ===");

    // 测试数据
    let test_data = b"This is test data for integrated storage system";
    let test_key = "integrated_test_001";

    // 1. 本地存储
    println!("\n1. 本地存储测试...");
    let local_metadata = storage.store_data(
        &format!("{}_local", test_key),
        test_data,
        StorageStrategy::LocalOnly,
        None,
    ).await?;
    
    println!("本地存储完成: {:?}", local_metadata);

    // 2. 区块链存储
    println!("\n2. 区块链存储测试...");
    let blockchain_metadata = storage.store_data(
        &format!("{}_blockchain", test_key),
        test_data,
        StorageStrategy::BlockchainOnly,
        Some(serde_json::json!({"type": "test", "strategy": "blockchain_only"})),
    ).await?;
    
    println!("区块链存储完成: {:?}", blockchain_metadata);

    // 3. 双重存储
    println!("\n3. 双重存储测试...");
    let dual_metadata = storage.store_data(
        &format!("{}_dual", test_key),
        test_data,
        StorageStrategy::Dual,
        Some(serde_json::json!({"type": "test", "strategy": "dual"})),
    ).await?;
    
    println!("双重存储完成: {:?}", dual_metadata);

    // 4. 自动策略存储
    println!("\n4. 自动策略存储测试...");
    let auto_metadata = storage.store_data(
        &format!("{}_auto", test_key),
        test_data,
        StorageStrategy::Auto,
        None,
    ).await?;
    
    println!("自动策略存储完成: {:?}", auto_metadata);

    // 5. 数据检索测试
    println!("\n5. 数据检索测试...");
    
    for key_suffix in ["local", "blockchain", "dual", "auto"] {
        let key = format!("{}_{}", test_key, key_suffix);
        match storage.retrieve_data(&key).await {
            Ok(data) => println!("检索成功 {}: {} bytes", key, data.len()),
            Err(e) => println!("检索失败 {}: {}", key, e),
        }
    }

    // 6. 数据完整性验证
    println!("\n6. 数据完整性验证...");
    let expected_hash = hex::encode(sha2::Sha256::digest(test_data));
    
    for key_suffix in ["local", "blockchain", "dual", "auto"] {
        let key = format!("{}_{}", test_key, key_suffix);
        match storage.verify_data(&key, &expected_hash).await {
            Ok(is_valid) => println!("数据完整性验证 {}: {}", key, if is_valid { "通过" } else { "失败" }),
            Err(e) => println!("数据完整性验证失败 {}: {}", key, e),
        }
    }

    // 7. 存储统计
    println!("\n7. 存储统计...");
    let stats = storage.get_storage_stats().await?;
    println!("本地存储项目数: {}", stats.local_items);
    println!("本地存储大小: {} bytes", stats.local_size);
    println!("总项目数: {}", stats.total_items);
    
    for (blockchain_type, blockchain_stat) in &stats.blockchain_stats {
        println!("{:?} 区块链统计:", blockchain_type);
        println!("  交易数: {}", blockchain_stat.total_transactions);
        println!("  数据大小: {} bytes", blockchain_stat.total_data_size);
        println!("  成功率: {:.2}%", blockchain_stat.success_rate * 100.0);
    }

    println!("\n=== 集成存储系统示例完成 ===");
    Ok(())
}
