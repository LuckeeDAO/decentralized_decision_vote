use shared_types::*;
use shared_utils::{crypto::verify_commitment, validation::*};
use chrono::Utc;

/// Validator for vote-related operations
pub struct VoteValidator;

impl Default for VoteValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl VoteValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate vote configuration
    pub fn validate_vote_config(&self, config: &VoteConfig) -> Result<(), VoteError> {
        // Validate title
        validate_not_empty(&config.title, "title")?;
        validate_string_length(&config.title, "title", Some(1), Some(200))?;
        
        // Validate description
        validate_not_empty(&config.description, "description")?;
        validate_string_length(&config.description, "description", Some(1), Some(1000))?;
        
        // Validate template_id
        validate_not_empty(&config.template_id, "template_id")?;
        
        // Validate template_params
        validate_not_null(&config.template_params, "template_params")?;
        
        // Validate durations
        if config.commitment_duration_hours == 0 {
            return Err(VoteError::InvalidConfig {
                message: "Commitment duration must be greater than 0".to_string(),
            });
        }
        
        if config.reveal_duration_hours == 0 {
            return Err(VoteError::InvalidConfig {
                message: "Reveal duration must be greater than 0".to_string(),
            });
        }
        
        // Check reasonable limits
        if config.commitment_duration_hours > 168 { // 1 week
            return Err(VoteError::InvalidConfig {
                message: "Commitment duration cannot exceed 168 hours (1 week)".to_string(),
            });
        }
        
        if config.reveal_duration_hours > 168 { // 1 week
            return Err(VoteError::InvalidConfig {
                message: "Reveal duration cannot exceed 168 hours (1 week)".to_string(),
            });
        }
        
        Ok(())
    }

    /// Validate that the vote is in commitment phase
    pub fn validate_commitment_phase(&self, vote: &Vote) -> Result<(), VoteError> {
        let now = Utc::now();
        
        if now < vote.commitment_start {
            return Err(VoteError::InvalidState {
                expected: "Commitment phase started".to_string(),
                actual: "Commitment phase not yet started".to_string(),
            });
        }
        
        if now > vote.commitment_end {
            return Err(VoteError::CommitmentPhaseNotActive);
        }
        
        Ok(())
    }

    /// Validate that the vote is in reveal phase
    pub fn validate_reveal_phase(&self, vote: &Vote) -> Result<(), VoteError> {
        let now = Utc::now();
        
        if now < vote.reveal_start {
            return Err(VoteError::InvalidState {
                expected: "Reveal phase started".to_string(),
                actual: "Reveal phase not yet started".to_string(),
            });
        }
        
        if now > vote.reveal_end {
            return Err(VoteError::RevealPhaseNotActive);
        }
        
        Ok(())
    }

    /// Validate commitment request
    pub fn validate_commitment(&self, request: &CommitRequest) -> Result<(), VoteError> {
        // Validate voter
        validate_not_empty(&request.voter, "voter")?;
        validate_string_length(&request.voter, "voter", Some(1), Some(100))?;
        
        // Validate commitment hash
        validate_not_empty(&request.commitment_hash, "commitment_hash")?;
        validate_string_length(&request.commitment_hash, "commitment_hash", Some(64), Some(64))?;
        
        // Validate salt
        validate_not_empty(&request.salt, "salt")?;
        validate_string_length(&request.salt, "salt", Some(1), Some(100))?;
        
        Ok(())
    }

    /// Validate reveal request
    pub fn validate_reveal(&self, request: &RevealRequest, commitment: &Commitment) -> Result<(), VoteError> {
        // Validate voter matches commitment
        if request.voter != commitment.voter {
            return Err(VoteError::InvalidReveal {
                message: "Voter does not match commitment".to_string(),
            });
        }
        
        // Validate salt matches commitment
        if request.salt != commitment.salt {
            return Err(VoteError::InvalidReveal {
                message: "Salt does not match commitment".to_string(),
            });
        }
        
        // Verify commitment
        let value_str = serde_json::to_string(&request.value)
            .map_err(|e| VoteError::InvalidReveal {
                message: format!("Invalid value format: {}", e),
            })?;
        
        if !verify_commitment(&value_str, &request.salt, &commitment.commitment_hash) {
            return Err(VoteError::InvalidReveal {
                message: "Reveal does not match commitment".to_string(),
            });
        }
        
        Ok(())
    }
}
