//! 区块链存储 trait 定义

use async_trait::async_trait;
use crate::{BlockchainType, NetworkConfig, StorageTransaction, StorageMetadata, StorageStats, Result};

/// 区块链存储接口
#[async_trait]
pub trait BlockchainStorage: Send + Sync {
    /// 存储数据到区块链
    async fn store_data(
        &self,
        key: &str,
        data: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction>;

    /// 从区块链检索数据
    async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>>;

    /// 验证数据完整性
    async fn verify_data(&self, key: &str, expected_hash: &str) -> Result<bool>;

    /// 获取存储元数据
    async fn get_metadata(&self, key: &str) -> Result<StorageMetadata>;

    /// 检查数据是否存在
    async fn exists(&self, key: &str) -> Result<bool>;

    /// 删除数据（如果支持）
    async fn delete_data(&self, key: &str) -> Result<StorageTransaction>;

    /// 获取存储统计信息
    async fn get_stats(&self) -> Result<StorageStats>;

    /// 获取区块链类型
    fn get_blockchain_type(&self) -> BlockchainType;

    /// 获取网络配置
    fn get_network_config(&self) -> &NetworkConfig;
}

/// 区块链客户端接口
#[async_trait]
pub trait BlockchainClient: Send + Sync {
    /// 连接到区块链网络
    async fn connect(&mut self) -> Result<()>;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<()>;

    /// 检查连接状态
    async fn is_connected(&self) -> Result<bool>;

    /// 获取当前区块高度
    async fn get_block_height(&self) -> Result<u64>;

    /// 获取账户余额
    async fn get_balance(&self, address: &str) -> Result<u128>;

    /// 估算交易费用
    async fn estimate_gas(&self, data: &[u8]) -> Result<u64>;

    /// 发送交易
    async fn send_transaction(&self, data: &[u8]) -> Result<String>;

    /// 等待交易确认
    async fn wait_for_confirmation(&self, tx_hash: &str) -> Result<StorageTransaction>;
}
