//! 区块链配置管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{BlockchainType, NetworkConfig};

/// 区块链存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// 默认区块链类型
    pub default_blockchain: BlockchainType,
    /// 网络配置映射
    pub networks: HashMap<String, NetworkConfig>,
    /// 私钥配置（加密存储）
    pub private_keys: HashMap<String, String>,
    /// 合约地址配置
    pub contract_addresses: HashMap<String, String>,
    /// 存储配置
    pub storage: StorageConfig,
    /// 重试配置
    pub retry: RetryConfig,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 最大数据大小（字节）
    pub max_data_size: u64,
    /// 数据分片大小
    pub chunk_size: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用加密
    pub enable_encryption: bool,
    /// 备份策略
    pub backup_strategy: BackupStrategy,
}

/// 备份策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStrategy {
    /// 不备份
    None,
    /// 单链备份
    Single,
    /// 多链备份
    Multiple(Vec<BlockchainType>),
    /// IPFS备份
    IPFS,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 延迟倍数
    pub delay_multiplier: f64,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        let mut networks = HashMap::new();
        
        // 以太坊主网配置
        networks.insert("ethereum_mainnet".to_string(), NetworkConfig {
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: Some(1),
            gas_price: Some("20".to_string()), // gwei
            gas_limit: Some(21000),
            timeout_seconds: 30,
            retry_attempts: 3,
        });

        // 以太坊测试网配置
        networks.insert("ethereum_goerli".to_string(), NetworkConfig {
            name: "Ethereum Goerli".to_string(),
            rpc_url: "https://goerli.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: Some(5),
            gas_price: Some("2".to_string()),
            gas_limit: Some(21000),
            timeout_seconds: 30,
            retry_attempts: 3,
        });

        // Solana 主网配置
        networks.insert("solana_mainnet".to_string(), NetworkConfig {
            name: "Solana Mainnet".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            chain_id: None,
            gas_price: None,
            gas_limit: None,
            timeout_seconds: 30,
            retry_attempts: 3,
        });

        Self {
            default_blockchain: BlockchainType::Ethereum,
            networks,
            private_keys: HashMap::new(),
            contract_addresses: HashMap::new(),
            storage: StorageConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_data_size: 1024 * 1024, // 1MB
            chunk_size: 32 * 1024,      // 32KB
            enable_compression: true,
            enable_encryption: true,
            backup_strategy: BackupStrategy::Single,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            delay_multiplier: 2.0,
        }
    }
}

impl BlockchainConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: BlockchainConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 获取网络配置
    pub fn get_network_config(&self, network_name: &str) -> Option<&NetworkConfig> {
        self.networks.get(network_name)
    }

    /// 添加网络配置
    pub fn add_network(&mut self, name: String, config: NetworkConfig) {
        self.networks.insert(name, config);
    }

    /// 设置私钥（应该加密存储）
    pub fn set_private_key(&mut self, network: &str, private_key: String) {
        // 在实际实现中，这里应该加密私钥
        self.private_keys.insert(network.to_string(), private_key);
    }

    /// 获取私钥
    pub fn get_private_key(&self, network: &str) -> Option<&String> {
        self.private_keys.get(network)
    }
}
