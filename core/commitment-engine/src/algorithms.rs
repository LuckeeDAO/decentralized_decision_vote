use async_trait::async_trait;
use sha2::{Sha256, Digest};
use hex;
use crate::engine::CommitmentError;

/// Trait for commitment algorithms
#[async_trait]
pub trait CommitmentAlgorithm: Send + Sync {
    /// Get the algorithm name
    fn name(&self) -> &'static str;
    
    /// Create a commitment hash
    async fn create_commitment(&self, value: &str, salt: &str) -> Result<String, CommitmentError>;
    
    /// Verify a commitment
    async fn verify_commitment(&self, value: &str, salt: &str, expected_hash: &str) -> Result<bool, CommitmentError>;
}

/// SHA256-based commitment algorithm
pub struct Sha256CommitmentAlgorithm;

impl Default for Sha256CommitmentAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256CommitmentAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommitmentAlgorithm for Sha256CommitmentAlgorithm {
    fn name(&self) -> &'static str {
        "sha256"
    }
    
    async fn create_commitment(&self, value: &str, salt: &str) -> Result<String, CommitmentError> {
        let combined = format!("{}:{}", value, salt);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
    
    async fn verify_commitment(&self, value: &str, salt: &str, expected_hash: &str) -> Result<bool, CommitmentError> {
        let actual_hash = self.create_commitment(value, salt).await?;
        Ok(actual_hash == expected_hash)
    }
}

/// Blake2b-based commitment algorithm (placeholder)
pub struct Blake2bCommitmentAlgorithm;

impl Default for Blake2bCommitmentAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl Blake2bCommitmentAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommitmentAlgorithm for Blake2bCommitmentAlgorithm {
    fn name(&self) -> &'static str {
        "blake2b"
    }
    
    async fn create_commitment(&self, value: &str, salt: &str) -> Result<String, CommitmentError> {
        // For now, use SHA256 as a placeholder
        // In a real implementation, this would use Blake2b
        let combined = format!("{}:{}", value, salt);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
    
    async fn verify_commitment(&self, value: &str, salt: &str, expected_hash: &str) -> Result<bool, CommitmentError> {
        let actual_hash = self.create_commitment(value, salt).await?;
        Ok(actual_hash == expected_hash)
    }
}

/// Registry for commitment algorithms
pub struct CommitmentAlgorithmRegistry {
    algorithms: std::collections::HashMap<String, std::sync::Arc<dyn CommitmentAlgorithm>>,
}

impl Default for CommitmentAlgorithmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitmentAlgorithmRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            algorithms: std::collections::HashMap::new(),
        };
        
        // Register default algorithms
        registry.register("sha256", std::sync::Arc::new(Sha256CommitmentAlgorithm::new()));
        registry.register("blake2b", std::sync::Arc::new(Blake2bCommitmentAlgorithm::new()));
        
        registry
    }
    
    pub fn register(&mut self, name: &str, algorithm: std::sync::Arc<dyn CommitmentAlgorithm>) {
        self.algorithms.insert(name.to_string(), algorithm);
    }
    
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn CommitmentAlgorithm>> {
        self.algorithms.get(name).cloned()
    }
    
    pub fn list(&self) -> Vec<String> {
        self.algorithms.keys().cloned().collect()
    }
}
