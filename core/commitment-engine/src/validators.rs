use crate::engine::{CommitmentData, CommitmentError};

/// Validator for commitment-related operations
pub struct CommitmentValidator;

impl Default for CommitmentValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitmentValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate commitment data
    pub fn validate_commitment_data(&self, data: &CommitmentData) -> Result<(), CommitmentError> {
        // Validate commitment hash
        if data.commitment_hash.is_empty() {
            return Err(CommitmentError::InvalidData {
                message: "Commitment hash cannot be empty".to_string(),
            });
        }

        // Validate salt
        if data.salt.is_empty() {
            return Err(CommitmentError::InvalidData {
                message: "Salt cannot be empty".to_string(),
            });
        }

        // Validate algorithm
        if data.algorithm.is_empty() {
            return Err(CommitmentError::InvalidData {
                message: "Algorithm cannot be empty".to_string(),
            });
        }

        // Validate supported algorithms
        let supported_algorithms = ["sha256", "blake2b"];
        if !supported_algorithms.contains(&data.algorithm.as_str()) {
            return Err(CommitmentError::InvalidData {
                message: format!("Unsupported algorithm: {}", data.algorithm),
            });
        }

        // Validate hash format (should be hex string)
        if !self.is_valid_hex(&data.commitment_hash) {
            return Err(CommitmentError::InvalidData {
                message: "Commitment hash must be a valid hex string".to_string(),
            });
        }

        Ok(())
    }

    /// Validate salt format
    pub fn validate_salt(&self, salt: &str) -> Result<(), CommitmentError> {
        if salt.is_empty() {
            return Err(CommitmentError::InvalidData {
                message: "Salt cannot be empty".to_string(),
            });
        }

        if salt.len() < 8 {
            return Err(CommitmentError::InvalidData {
                message: "Salt must be at least 8 characters long".to_string(),
            });
        }

        if salt.len() > 256 {
            return Err(CommitmentError::InvalidData {
                message: "Salt cannot exceed 256 characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate commitment hash format
    pub fn validate_commitment_hash(&self, hash: &str) -> Result<(), CommitmentError> {
        if hash.is_empty() {
            return Err(CommitmentError::InvalidData {
                message: "Commitment hash cannot be empty".to_string(),
            });
        }

        if !self.is_valid_hex(hash) {
            return Err(CommitmentError::InvalidData {
                message: "Commitment hash must be a valid hex string".to_string(),
            });
        }

        // SHA256 produces 64-character hex strings
        if hash.len() != 64 {
            return Err(CommitmentError::InvalidData {
                message: "Commitment hash must be 64 characters long (SHA256)".to_string(),
            });
        }

        Ok(())
    }

    /// Check if a string is valid hex
    fn is_valid_hex(&self, s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_hexdigit())
    }
}
