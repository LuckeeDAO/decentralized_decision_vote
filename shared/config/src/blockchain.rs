use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub network: String,
    pub rpc_url: String,
    pub private_key: Option<String>,
    pub contract_address: Option<String>,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/your-project-id".to_string(),
            private_key: None,
            contract_address: None,
            gas_limit: 100000,
            gas_price: 20000000000, // 20 gwei
        }
    }
}

impl BlockchainConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            network: std::env::var("BLOCKCHAIN_NETWORK")
                .unwrap_or_else(|_| "mainnet".to_string()),
            rpc_url: std::env::var("BLOCKCHAIN_RPC_URL")
                .unwrap_or_else(|_| "https://mainnet.infura.io/v3/your-project-id".to_string()),
            private_key: std::env::var("BLOCKCHAIN_PRIVATE_KEY").ok(),
            contract_address: std::env::var("BLOCKCHAIN_CONTRACT_ADDRESS").ok(),
            gas_limit: std::env::var("BLOCKCHAIN_GAS_LIMIT")
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .unwrap_or(100000),
            gas_price: std::env::var("BLOCKCHAIN_GAS_PRICE")
                .unwrap_or_else(|_| "20000000000".to_string())
                .parse()
                .unwrap_or(20000000000),
        })
    }
}
