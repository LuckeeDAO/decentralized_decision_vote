//! 区块链存储抽象层
//!
//! 提供统一的区块链存储接口，支持多种区块链网络
//! 包括以太坊、Solana、Cosmos等主流区块链

pub mod config;
pub mod ethereum;
pub mod solana;
pub mod cosmos;
pub mod archway;
pub mod injective;
pub mod avalanche;
pub mod sui;
pub mod error;
pub mod traits;
pub mod manager;

pub use config::BlockchainConfig;
pub use error::{BlockchainError, Result};
pub use traits::{BlockchainStorage, BlockchainClient};
pub use manager::BlockchainManager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 支持的区块链类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BlockchainType {
    Ethereum,
    Solana,
    Cosmos,
    Polygon,
    Arbitrum,
    Optimism,
    BSC,
    Archway,
    Injective,
    Avalanche,
    Sui,
}

/// 区块链网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub gas_price: Option<String>,
    pub gas_limit: Option<u64>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// 存储交易信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTransaction {
    pub tx_hash: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub status: TransactionStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data_hash: String,
    pub storage_key: String,
}

/// 交易状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

/// 存储数据元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub key: String,
    pub data_hash: String,
    pub size: u64,
    pub blockchain_type: BlockchainType,
    pub network: String,
    pub tx_hash: String,
    pub block_number: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
}

/// 区块链存储统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_transactions: u64,
    pub total_data_size: u64,
    pub average_gas_used: f64,
    pub success_rate: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub by_network: HashMap<String, NetworkStats>,
}

/// 网络统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub transaction_count: u64,
    pub total_gas_used: u64,
    pub success_count: u64,
    pub failure_count: u64,
}
