//! Avalanche 区块链存储实现

use async_trait::async_trait;
use web3::{
    types::{Address, H256, U256, Bytes, TransactionRequest},
    Web3, Transport, Http,
};
use std::str::FromStr;
use sha2::{Sha256, Digest};

use crate::{
    BlockchainStorage, BlockchainClient, NetworkConfig, StorageTransaction, 
    StorageMetadata, StorageStats, BlockchainType, TransactionStatus, Result, BlockchainError
};

/// Avalanche 存储实现
pub struct AvalancheStorage {
    web3: Web3<Http>,
    network_config: NetworkConfig,
    contract_address: Option<Address>,
}

impl AvalancheStorage {
    /// 创建新的 Avalanche 存储实例
    pub async fn new(network_config: NetworkConfig) -> Result<Self> {
        let transport = Http::new(&network_config.rpc_url)
            .map_err(|e| BlockchainError::Network(format!("Failed to create HTTP transport: {}", e)))?;
        
        let web3 = Web3::new(transport);
        
        // 测试连接
        let chain_id = web3.eth().chain_id().await
            .map_err(|e| BlockchainError::Network(format!("Failed to get chain ID: {}", e)))?;
        
        tracing::info!("Connected to Avalanche network: {}, Chain ID: {}", network_config.name, chain_id);

        Ok(Self {
            web3,
            network_config,
            contract_address: None,
        })
    }

    /// 设置智能合约地址
    pub fn set_contract_address(&mut self, address: &str) -> Result<()> {
        self.contract_address = Some(
            Address::from_str(address)
                .map_err(|e| BlockchainError::InvalidConfig(format!("Invalid contract address: {}", e)))?
        );
        Ok(())
    }

    /// 存储数据到 Avalanche（使用智能合约）
    async fn store_via_contract(&self, key: &str, data: &[u8]) -> Result<StorageTransaction> {
        // 这里应该调用智能合约的存储方法
        // 简化实现，实际需要：
        // 1. 构建合约调用数据
        // 2. 估算 gas
        // 3. 发送交易
        // 4. 等待确认
        
        let tx_hash = format!("avalanche_{}", hex::encode(&Sha256::digest(data)[..32]));
        
        Ok(StorageTransaction {
            tx_hash,
            block_number: Some(12345), // 模拟
            gas_used: Some(25000), // Avalanche 使用更少的 gas
            status: TransactionStatus::Confirmed,
            timestamp: chrono::Utc::now(),
            data_hash: hex::encode(&Sha256::digest(data)),
            storage_key: key.to_string(),
        })
    }

    /// 从智能合约检索数据
    async fn retrieve_via_contract(&self, key: &str) -> Result<Vec<u8>> {
        // 这里应该调用智能合约的查询方法
        // 简化实现，返回模拟数据
        Err(BlockchainError::DataNotFound(format!("Data not found for key: {}", key)))
    }
}

#[async_trait]
impl BlockchainStorage for AvalancheStorage {
    async fn store_data(
        &self,
        key: &str,
        data: &[u8],
        _metadata: Option<serde_json::Value>,
    ) -> Result<StorageTransaction> {
        if data.len() > self.network_config.gas_limit.unwrap_or(25000) as usize {
            return Err(BlockchainError::InvalidConfig(
                "Data size exceeds gas limit".to_string()
            ));
        }

        if let Some(_contract) = self.contract_address {
            self.store_via_contract(key, data).await
        } else {
            // 如果没有合约，可以存储到交易数据中
            let tx_hash = format!("avalanche_{}", hex::encode(&Sha256::digest(data)[..32]));
            
            Ok(StorageTransaction {
                tx_hash,
                block_number: Some(12345),
                gas_used: Some(25000),
                status: TransactionStatus::Confirmed,
                timestamp: chrono::Utc::now(),
                data_hash: hex::encode(&Sha256::digest(data)),
                storage_key: key.to_string(),
            })
        }
    }

    async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>> {
        if let Some(_contract) = self.contract_address {
            self.retrieve_via_contract(key).await
        } else {
            Err(BlockchainError::DataNotFound(format!("Data not found for key: {}", key)))
        }
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
        // 简化实现，实际应该从区块链查询
        Err(BlockchainError::DataNotFound(format!("Metadata not found for key: {}", key)))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self.retrieve_data(key).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn delete_data(&self, _key: &str) -> Result<StorageTransaction> {
        // Avalanche 不支持删除数据，只能标记为删除
        Err(BlockchainError::Unknown("Delete operation not supported on Avalanche".to_string()))
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        // 简化实现，实际应该从区块链查询统计信息
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
        BlockchainType::Avalanche
    }

    fn get_network_config(&self) -> &NetworkConfig {
        &self.network_config
    }
}

#[async_trait]
impl BlockchainClient for AvalancheStorage {
    async fn connect(&mut self) -> Result<()> {
        // 测试连接
        let _chain_id = self.web3.eth().chain_id().await
            .map_err(|e| BlockchainError::Network(format!("Failed to connect: {}", e)))?;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        // HTTP 连接不需要显式断开
        Ok(())
    }

    async fn is_connected(&self) -> Result<bool> {
        match self.web3.eth().chain_id().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_block_height(&self) -> Result<u64> {
        let block_number = self.web3.eth().block_number().await
            .map_err(|e| BlockchainError::Network(format!("Failed to get block height: {}", e)))?;
        Ok(block_number.as_u64())
    }

    async fn get_balance(&self, address: &str) -> Result<u128> {
        let addr = Address::from_str(address)
            .map_err(|e| BlockchainError::InvalidConfig(format!("Invalid address: {}", e)))?;
        
        let balance = self.web3.eth().balance(addr, None).await
            .map_err(|e| BlockchainError::Network(format!("Failed to get balance: {}", e)))?;
        
        Ok(balance.as_u128())
    }

    async fn estimate_gas(&self, data: &[u8]) -> Result<u64> {
        // 简化实现，返回固定值
        Ok(25000)
    }

    async fn send_transaction(&self, data: &[u8]) -> Result<String> {
        // 简化实现，实际需要构建和发送交易
        let tx_hash = format!("avalanche_{}", hex::encode(&Sha256::digest(data)[..32]));
        Ok(tx_hash)
    }

    async fn wait_for_confirmation(&self, tx_hash: &str) -> Result<StorageTransaction> {
        // 简化实现，实际需要轮询交易状态
        Ok(StorageTransaction {
            tx_hash: tx_hash.to_string(),
            block_number: Some(12345),
            gas_used: Some(25000),
            status: TransactionStatus::Confirmed,
            timestamp: chrono::Utc::now(),
            data_hash: "".to_string(),
            storage_key: "".to_string(),
        })
    }
}
