//! 区块链管理器
//!
//! 负责管理多个区块链客户端和存储实例

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::{
    BlockchainConfig, BlockchainType, NetworkConfig, 
    BlockchainStorage, BlockchainClient, StorageTransaction, 
    StorageMetadata, StorageStats, Result
};
use crate::ethereum::EthereumStorage;
use crate::solana::SolanaStorage;
use crate::cosmos::CosmosStorage;
use crate::archway::ArchwayStorage;
use crate::injective::InjectiveStorage;
use crate::avalanche::AvalancheStorage;
use crate::sui::SuiStorage;

/// 区块链管理器
pub struct BlockchainManager {
    config: BlockchainConfig,
    storages: Arc<RwLock<HashMap<BlockchainType, Box<dyn BlockchainStorage>>>>,
    clients: Arc<RwLock<HashMap<BlockchainType, Box<dyn BlockchainClient>>>>,
}

impl BlockchainManager {
    /// 创建新的区块链管理器
    pub fn new(config: BlockchainConfig) -> Self {
        Self {
            config,
            storages: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 初始化所有配置的区块链客户端
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing blockchain manager...");

        // 初始化以太坊客户端
        if let Some(eth_config) = self.config.networks.get("ethereum_mainnet") {
            let eth_storage = EthereumStorage::new(eth_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Ethereum, Box::new(eth_storage));
        }

        // 初始化 Solana 客户端
        if let Some(sol_config) = self.config.networks.get("solana_mainnet") {
            let sol_storage = SolanaStorage::new(sol_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Solana, Box::new(sol_storage));
        }

        // 初始化 Cosmos 客户端
        if let Some(cosmos_config) = self.config.networks.get("cosmos_mainnet") {
            let cosmos_storage = CosmosStorage::new(cosmos_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Cosmos, Box::new(cosmos_storage));
        }

        // 初始化 Archway 客户端
        if let Some(archway_config) = self.config.networks.get("archway_mainnet") {
            let archway_storage = ArchwayStorage::new(archway_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Archway, Box::new(archway_storage));
        }

        // 初始化 Injective 客户端
        if let Some(injective_config) = self.config.networks.get("injective_mainnet") {
            let injective_storage = InjectiveStorage::new(injective_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Injective, Box::new(injective_storage));
        }

        // 初始化 Avalanche 客户端
        if let Some(avalanche_config) = self.config.networks.get("avalanche_mainnet") {
            let avalanche_storage = AvalancheStorage::new(avalanche_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Avalanche, Box::new(avalanche_storage));
        }

        // 初始化 Sui 客户端
        if let Some(sui_config) = self.config.networks.get("sui_mainnet") {
            let sui_storage = SuiStorage::new(sui_config.clone()).await?;
            let mut storages = self.storages.write().await;
            storages.insert(BlockchainType::Sui, Box::new(sui_storage));
        }

        tracing::info!("Blockchain manager initialized successfully");
        Ok(())
    }

    /// 获取指定类型的区块链存储
    pub async fn get_storage(&self, blockchain_type: &BlockchainType) -> Result<Box<dyn BlockchainStorage>> {
        let storages = self.storages.read().await;
        if let Some(storage) = storages.get(blockchain_type) {
            // 这里需要克隆 storage，实际实现中可能需要调整
            Err(BlockchainError::Unknown("Storage cloning not implemented".to_string()))
        } else {
            Err(BlockchainError::DataNotFound(format!("Storage for {:?} not found", blockchain_type)))
        }
    }

    /// 获取默认区块链存储
    pub async fn get_default_storage(&self) -> Result<Box<dyn BlockchainStorage>> {
        self.get_storage(&self.config.default_blockchain).await
    }

    /// 存储数据到指定区块链
    pub async fn store_data(
        &self,
        blockchain_type: &BlockchainType,
        key: &str,
        data: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction> {
        let storage = self.get_storage(blockchain_type).await?;
        storage.store_data(key, data, metadata).await
    }

    /// 从指定区块链检索数据
    pub async fn retrieve_data(&self, blockchain_type: &BlockchainType, key: &str) -> Result<Vec<u8>> {
        let storage = self.get_storage(blockchain_type).await?;
        storage.retrieve_data(key).await
    }

    /// 获取所有区块链的统计信息
    pub async fn get_all_stats(&self) -> Result<HashMap<BlockchainType, StorageStats>> {
        let mut stats = HashMap::new();
        let storages = self.storages.read().await;

        for (blockchain_type, storage) in storages.iter() {
            match storage.get_stats().await {
                Ok(stat) => {
                    stats.insert(blockchain_type.clone(), stat);
                }
                Err(e) => {
                    tracing::warn!("Failed to get stats for {:?}: {}", blockchain_type, e);
                }
            }
        }

        Ok(stats)
    }

    /// 添加新的区块链存储
    pub async fn add_storage(
        &self,
        blockchain_type: BlockchainType,
        storage: Box<dyn BlockchainStorage>,
    ) -> Result<()> {
        let mut storages = self.storages.write().await;
        storages.insert(blockchain_type, storage);
        Ok(())
    }

    /// 移除区块链存储
    pub async fn remove_storage(&self, blockchain_type: &BlockchainType) -> Result<()> {
        let mut storages = self.storages.write().await;
        storages.remove(blockchain_type);
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &BlockchainConfig {
        &self.config
    }

    /// 更新配置
    pub async fn update_config(&mut self, config: BlockchainConfig) -> Result<()> {
        self.config = config;
        // 重新初始化所有客户端
        self.initialize().await?;
        Ok(())
    }
}

#[async_trait]
impl BlockchainStorage for BlockchainManager {
    async fn store_data(
        &self,
        key: &str,
        data: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction> {
        self.store_data(&self.config.default_blockchain, key, data, metadata).await
    }

    async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>> {
        self.retrieve_data(&self.config.default_blockchain, key).await
    }

    async fn verify_data(&self, key: &str, expected_hash: &str) -> Result<bool> {
        let storage = self.get_default_storage().await?;
        storage.verify_data(key, expected_hash).await
    }

    async fn get_metadata(&self, key: &str) -> Result<StorageMetadata> {
        let storage = self.get_default_storage().await?;
        storage.get_metadata(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let storage = self.get_default_storage().await?;
        storage.exists(key).await
    }

    async fn delete_data(&self, key: &str) -> Result<StorageTransaction> {
        let storage = self.get_default_storage().await?;
        storage.delete_data(key).await
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        let storage = self.get_default_storage().await?;
        storage.get_stats().await
    }

    fn get_blockchain_type(&self) -> BlockchainType {
        self.config.default_blockchain.clone()
    }

    fn get_network_config(&self) -> &NetworkConfig {
        // 这里需要根据默认区块链类型返回对应的网络配置
        // 简化实现，实际应该查找对应的网络配置
        self.config.networks.values().next().unwrap()
    }
}
