//! 区块链存储错误定义

use thiserror::Error;

/// 区块链存储错误类型
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Data not found: {0}")]
    DataNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Web3 error: {0}")]
    Web3(String),

    #[error("Solana error: {0}")]
    Solana(String),

    #[error("Cosmos error: {0}")]
    Cosmos(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

impl From<web3::Error> for BlockchainError {
    fn from(err: web3::Error) -> Self {
        BlockchainError::Web3(err.to_string())
    }
}

impl From<solana_client::client_error::ClientError> for BlockchainError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        BlockchainError::Solana(err.to_string())
    }
}

impl From<cosmwasm_std::StdError> for BlockchainError {
    fn from(err: cosmwasm_std::StdError) -> Self {
        BlockchainError::Cosmos(err.to_string())
    }
}
