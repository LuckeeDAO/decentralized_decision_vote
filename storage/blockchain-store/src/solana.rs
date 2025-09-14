//! Solana 区块链存储实现

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
    system_instruction,
};
use std::str::FromStr;
use sha2::{Sha256, Digest};

use crate::{
    BlockchainStorage, BlockchainClient, NetworkConfig, StorageTransaction, 
    StorageMetadata, StorageStats, BlockchainType, TransactionStatus, Result, BlockchainError
};

/// Solana 存储实现
pub struct SolanaStorage {
    client: RpcClient,
    network_config: NetworkConfig,
    program_id: Option<Pubkey>,
}

impl SolanaStorage {
    /// 创建新的 Solana 存储实例
    pub async fn new(network_config: NetworkConfig) -> Result<Self> {
        let client = RpcClient::new(network_config.rpc_url.clone());
        
        // 测试连接
        let version = client.get_version()
            .map_err(|e| BlockchainError::Network(format!("Failed to connect to Solana: {}", e)))?;
        
        tracing::info!("Connected to Solana network: {}, Version: {}", network_config.name, version.solana_core);

        Ok(Self {
            client,
            network_config,
            program_id: None,
        })
    }

    /// 设置程序 ID
    pub fn set_program_id(&mut self, program_id: &str) -> Result<()> {
        self.program_id = Some(
            Pubkey::from_str(program_id)
                .map_err(|e| BlockchainError::InvalidConfig(format!("Invalid program ID: {}", e)))?
        );
        Ok(())
    }

    /// 存储数据到 Solana 账户
    async fn store_to_account(&self, key: &str, data: &[u8]) -> Result<StorageTransaction> {
        // 简化实现，实际需要：
        // 1. 创建或找到存储账户
        // 2. 构建存储指令
        // 3. 发送交易
        // 4. 等待确认
        
        let tx_hash = format!("{}", hex::encode(&Sha256::digest(data)[..32]));
        
        Ok(StorageTransaction {
            tx_hash,
            block_number: Some(12345), // Solana 使用 slot
            gas_used: Some(5000), // Solana 使用 compute units
            status: TransactionStatus::Confirmed,
            timestamp: chrono::Utc::now(),
            data_hash: hex::encode(&Sha256::digest(data)),
            storage_key: key.to_string(),
        })
    }

    /// 从 Solana 账户检索数据
    async fn retrieve_from_account(&self, key: &str) -> Result<Vec<u8>> {
        // 简化实现，实际需要查询账户数据
        Err(BlockchainError::DataNotFound(format!("Data not found for key: {}", key)))
    }
}

#[async_trait]
impl BlockchainStorage for SolanaStorage {
    async fn store_data(
        &self,
        key: &str,
        data: &[u8],
        _metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction> {
        // Solana 账户数据限制
        if data.len() > 10 * 1024 * 1024 { // 10MB 限制
            return Err(BlockchainError::InvalidConfig(
                "Data size exceeds Solana account limit".to_string()
            ));
        }

        self.store_to_account(key, data).await
    }

    async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>> {
        self.retrieve_from_account(key).await
    }

    async fn verify_data(&self, key: &str, expected_hash: &str) -> Result<bool> {
        match self.retrieve_data(key).await {
            Ok(data) => {
                let actual_hash = hex::encode(&Sha256::digest(&data));
                Ok(actual_hash == expected_hash)
            }
            Err(_) => Ok(false),
        }
    }

    async fn get_metadata(&self, key: &str) -> Result<StorageMetadata> {
        // 简化实现，实际应该从 Solana 查询
        Err(BlockchainError::DataNotFound(format!("Metadata not found for key: {}", key)))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self.retrieve_data(key).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn delete_data(&self, _key: &str) -> Result<StorageTransaction> {
        // Solana 不支持删除账户数据，只能清零
        Err(BlockchainError::Unknown("Delete operation not supported on Solana".to_string()))
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        // 简化实现，实际应该从 Solana 查询统计信息
        Ok(StorageStats {
            total_transactions: 0,
            total_data_size: 0,
            average_gas_used: 0.0,
            success_rate: 1.0,
            last_updated: chrono::Utc::now(),
            by_network: std::collections::HashMap::new(),
        })
    }

    fn get_blockchain_type(&self) -> BlockchainType {
        BlockchainType::Solana
    }

    fn get_network_config(&self) -> &NetworkConfig {
        &self.network_config
    }
}

#[async_trait]
impl BlockchainClient for SolanaStorage {
    async fn connect(&mut self) -> Result<()> {
        // 测试连接
        let _version = self.client.get_version()
            .map_err(|e| BlockchainError::Network(format!("Failed to connect: {}", e)))?;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        // RPC 客户端不需要显式断开
        Ok(())
    }

    async fn is_connected(&self) -> Result<bool> {
        match self.client.get_version() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_block_height(&self) -> Result<u64> {
        let slot = self.client.get_slot()
            .map_err(|e| BlockchainError::Network(format!("Failed to get slot: {}", e)))?;
        Ok(slot)
    }

    async fn get_balance(&self, address: &str) -> Result<u128> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| BlockchainError::InvalidConfig(format!("Invalid address: {}", e)))?;
        
        let balance = self.client.get_balance(&pubkey)
            .map_err(|e| BlockchainError::Network(format!("Failed to get balance: {}", e)))?;
        
        Ok(balance)
    }

    async fn estimate_gas(&self, data: &[u8]) -> Result<u64> {
        // 简化实现，基于数据大小估算 compute units
        Ok(data.len() as u64 * 100)
    }

    async fn send_transaction(&self, data: &[u8]) -> Result<String> {
        // 简化实现，实际需要构建和发送交易
        let tx_hash = format!("{}", hex::encode(&Sha256::digest(data)[..32]));
        Ok(tx_hash)
    }

    async fn wait_for_confirmation(&self, tx_hash: &str) -> Result<StorageTransaction> {
        // 简化实现，实际需要轮询交易状态
        Ok(StorageTransaction {
            tx_hash: tx_hash.to_string(),
            block_number: Some(12345),
            gas_used: Some(5000),
            status: TransactionStatus::Confirmed,
            timestamp: chrono::Utc::now(),
            data_hash: "".to_string(),
            storage_key: "".to_string(),
        })
    }
}
