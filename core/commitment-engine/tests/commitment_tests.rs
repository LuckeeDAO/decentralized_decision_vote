use std::sync::Arc;
use commitment_engine::*;
use sha2::{Sha256, Digest};
use hex;

// Mock implementation for testing
struct MockCommitmentAlgorithm {
    name: &'static str,
}

impl MockCommitmentAlgorithm {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl CommitmentAlgorithm for MockCommitmentAlgorithm {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn create_commitment(&self, value: &str, salt: &str) -> Result<String, CommitmentError> {
        // Simple mock implementation: hash value + salt using SHA256
        let combined = format!("{}:{}", value, salt);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }

    async fn verify_commitment(&self, value: &str, salt: &str, commitment_hash: &str) -> Result<bool, CommitmentError> {
        let expected_hash = self.create_commitment(value, salt).await?;
        Ok(expected_hash == commitment_hash)
    }
}

#[tokio::test]
async fn test_create_commitment_success() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    let result = engine.create_commitment("yes", "test_voter").await;
    assert!(result.is_ok());

    let commitment_data = result.unwrap();
    assert!(!commitment_data.commitment_hash.is_empty());
    assert!(!commitment_data.salt.is_empty());
    assert_eq!(commitment_data.algorithm, "sha256");
}

#[tokio::test]
async fn test_verify_commitment_success() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    // Create a commitment
    let commitment_data = engine.create_commitment("yes", "test_voter").await.unwrap();

    // Verify the commitment
    let result = engine.verify_commitment(
        "yes",
        &commitment_data.salt,
        &commitment_data.commitment_hash,
    ).await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_verify_commitment_failure() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    // Create a commitment
    let commitment_data = engine.create_commitment("yes", "test_voter").await.unwrap();

    // Try to verify with wrong value
    let result = engine.verify_commitment(
        "no", // Wrong value
        &commitment_data.salt,
        &commitment_data.commitment_hash,
    ).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_verify_commitment_wrong_salt() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    // Create a commitment
    let commitment_data = engine.create_commitment("yes", "test_voter").await.unwrap();

    // Try to verify with wrong salt
    let result = engine.verify_commitment(
        "yes",
        "wrong_salt",
        &commitment_data.commitment_hash,
    ).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_validate_commitment_data() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    // Create valid commitment data
    let commitment_data = engine.create_commitment("yes", "test_voter").await.unwrap();

    // Validate the data
    let result = engine.validate_commitment_data(&commitment_data);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_commitment_data_invalid() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    // Create invalid commitment data
    let invalid_data = CommitmentData {
        commitment_hash: "".to_string(), // Empty hash
        salt: "test_salt".to_string(),
        algorithm: "sha256".to_string(),
        created_at: chrono::Utc::now(),
    };

    // Validate the data
    let result = engine.validate_commitment_data(&invalid_data);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_supported_algorithms() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    let algorithms = engine.get_supported_algorithms();
    assert!(algorithms.contains(&"sha256".to_string()));
    assert!(algorithms.contains(&"blake2b".to_string()));
}

#[tokio::test]
async fn test_commitment_consistency() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    let value = "test_value";
    let voter = "test_voter";

    // Create multiple commitments with the same value
    let commitment1 = engine.create_commitment(value, voter).await.unwrap();
    let commitment2 = engine.create_commitment(value, voter).await.unwrap();

    // They should have different hashes (due to different salts)
    assert_ne!(commitment1.commitment_hash, commitment2.commitment_hash);
    assert_ne!(commitment1.salt, commitment2.salt);

    // But both should verify correctly
    let verify1 = engine.verify_commitment(value, &commitment1.salt, &commitment1.commitment_hash).await.unwrap();
    let verify2 = engine.verify_commitment(value, &commitment2.salt, &commitment2.commitment_hash).await.unwrap();

    assert!(verify1);
    assert!(verify2);
}

#[tokio::test]
async fn test_commitment_serialization() {
    let algorithm = Arc::new(MockCommitmentAlgorithm::new("sha256"));
    let engine = CommitmentEngine::new(algorithm);

    let commitment_data = engine.create_commitment("yes", "test_voter").await.unwrap();

    // Test serialization
    let serialized = serde_json::to_string(&commitment_data).unwrap();
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: CommitmentData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.commitment_hash, commitment_data.commitment_hash);
    assert_eq!(deserialized.salt, commitment_data.salt);
    assert_eq!(deserialized.algorithm, commitment_data.algorithm);
}
