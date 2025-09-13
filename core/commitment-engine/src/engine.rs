use std::sync::Arc;
use tracing::{info, debug};
use shared_utils::generate_salt;

use crate::algorithms::CommitmentAlgorithm;
use crate::validators::CommitmentValidator;

/// Commitment engine for handling vote commitments
pub struct CommitmentEngine {
    algorithm: Arc<dyn CommitmentAlgorithm>,
    validator: Arc<CommitmentValidator>,
}

impl CommitmentEngine {
    pub fn new(algorithm: Arc<dyn CommitmentAlgorithm>) -> Self {
        Self {
            algorithm,
            validator: Arc::new(CommitmentValidator::new()),
        }
    }

    /// Create a commitment for a vote value
    pub async fn create_commitment(&self, value: &str, voter: &str) -> Result<CommitmentData, CommitmentError> {
        info!("Creating commitment for voter: {}", voter);
        
        // Generate salt
        let salt = generate_salt();
        
        // Create commitment using the algorithm
        let commitment_hash = self.algorithm.create_commitment(value, &salt).await?;
        
        let commitment_data = CommitmentData {
            commitment_hash,
            salt,
            algorithm: self.algorithm.name().to_string(),
            created_at: chrono::Utc::now(),
        };
        
        debug!("Commitment created successfully for voter: {}", voter);
        Ok(commitment_data)
    }

    /// Verify a commitment against a revealed value
    pub async fn verify_commitment(&self, value: &str, salt: &str, commitment_hash: &str) -> Result<bool, CommitmentError> {
        debug!("Verifying commitment");
        
        // Use the algorithm to verify
        let is_valid = self.algorithm.verify_commitment(value, salt, commitment_hash).await?;
        
        debug!("Commitment verification result: {}", is_valid);
        Ok(is_valid)
    }

    /// Validate commitment data
    pub fn validate_commitment_data(&self, data: &CommitmentData) -> Result<(), CommitmentError> {
        self.validator.validate_commitment_data(data)
    }

    /// Get supported algorithms
    pub fn get_supported_algorithms(&self) -> Vec<String> {
        vec![
            "sha256".to_string(),
            "blake2b".to_string(),
        ]
    }
}

/// Data structure for commitment information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommitmentData {
    pub commitment_hash: String,
    pub salt: String,
    pub algorithm: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Error types for commitment operations
#[derive(thiserror::Error, Debug)]
pub enum CommitmentError {
    #[error("Invalid commitment data: {message}")]
    InvalidData { message: String },
    
    #[error("Commitment verification failed: {message}")]
    VerificationFailed { message: String },
    
    #[error("Algorithm error: {message}")]
    AlgorithmError { message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
