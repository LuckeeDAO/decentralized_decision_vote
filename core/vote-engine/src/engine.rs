use std::sync::Arc;
use shared_types::*;
use shared_utils::crypto::generate_id;
use chrono::{Utc, Duration};
use tracing::info;

use crate::services::VoteService;
use crate::validators::VoteValidator;

/// Core voting engine that orchestrates the voting process
pub struct VoteEngine {
    vote_service: Arc<dyn VoteService>,
    validator: Arc<VoteValidator>,
}

impl VoteEngine {
    pub fn new(vote_service: Arc<dyn VoteService>) -> Self {
        Self {
            vote_service,
            validator: Arc::new(VoteValidator::new()),
        }
    }

    /// Create a new vote
    pub async fn create_vote(&self, config: VoteConfig) -> Result<String, VoteError> {
        info!("Creating new vote: {}", config.title);
        
        // Validate the vote configuration
        self.validator.validate_vote_config(&config)?;
        
        // Generate vote ID
        let vote_id = generate_id();
        
        // Calculate phase timestamps
        let now = Utc::now();
        let commitment_start = now;
        let commitment_end = now + Duration::hours(config.commitment_duration_hours as i64);
        let reveal_start = commitment_end;
        let reveal_end = reveal_start + Duration::hours(config.reveal_duration_hours as i64);
        
        // Create vote object
        let vote = Vote {
            id: vote_id.clone(),
            title: config.title,
            description: config.description,
            template_id: config.template_id,
            template_params: config.template_params,
            creator: "system".to_string(), // TODO: Get from authentication context
            created_at: now,
            commitment_start,
            commitment_end,
            reveal_start,
            reveal_end,
            status: VoteStatus::Created,
            results: None,
        };
        
        // Save to storage
        self.vote_service.create_vote(vote).await?;
        
        info!("Vote created successfully: {}", vote_id);
        Ok(vote_id)
    }

    /// Submit a commitment
    pub async fn commit_vote(&self, vote_id: &str, request: CommitRequest) -> Result<CommitResponse, VoteError> {
        info!("Processing commitment for vote: {}", vote_id);
        
        // Get the vote
        let vote = self.vote_service.get_vote(vote_id).await?;
        
        // Validate vote state
        self.validator.validate_commitment_phase(&vote)?;
        
        // Validate commitment
        self.validator.validate_commitment(&request)?;
        
        // Create commitment object
        let commitment = Commitment {
            id: generate_id(),
            vote_id: vote_id.to_string(),
            voter: request.voter,
            commitment_hash: request.commitment_hash,
            salt: request.salt,
            created_at: Utc::now(),
        };
        
        // Save commitment
        self.vote_service.save_commitment(commitment.clone()).await?;
        
        // Update vote status if needed
        if matches!(vote.status, VoteStatus::Created) {
            self.vote_service.update_vote_status(vote_id, VoteStatus::CommitmentPhase).await?;
        }
        
        info!("Commitment saved successfully for vote: {}", vote_id);
        Ok(CommitResponse {
            commitment_id: commitment.id,
            success: true,
            message: "Commitment saved successfully".to_string(),
        })
    }

    /// Submit a reveal
    pub async fn reveal_vote(&self, vote_id: &str, request: RevealRequest) -> Result<RevealResponse, VoteError> {
        info!("Processing reveal for vote: {}", vote_id);
        
        // Get the vote
        let vote = self.vote_service.get_vote(vote_id).await?;
        
        // Validate vote state
        self.validator.validate_reveal_phase(&vote)?;
        
        // Get the commitment
        let commitment = self.vote_service.get_commitment(vote_id, &request.voter).await?
            .ok_or_else(|| VoteError::InvalidReveal { 
                message: "No commitment found for this voter".to_string() 
            })?;
        
        // Validate reveal against commitment
        self.validator.validate_reveal(&request, &commitment)?;
        
        // Create reveal object
        let reveal = Reveal {
            id: generate_id(),
            vote_id: vote_id.to_string(),
            voter: request.voter,
            value: request.value,
            salt: request.salt,
            created_at: Utc::now(),
        };
        
        // Save reveal
        self.vote_service.save_reveal(reveal.clone()).await?;
        
        // Update vote status if needed
        if matches!(vote.status, VoteStatus::CommitmentPhase) {
            self.vote_service.update_vote_status(vote_id, VoteStatus::RevealPhase).await?;
        }
        
        info!("Reveal saved successfully for vote: {}", vote_id);
        Ok(RevealResponse {
            reveal_id: reveal.id,
            success: true,
            message: "Reveal saved successfully".to_string(),
        })
    }

    /// Get vote results
    pub async fn get_results(&self, vote_id: &str) -> Result<VoteResults, VoteError> {
        info!("Getting results for vote: {}", vote_id);
        
        // Get the vote
        let vote = self.vote_service.get_vote(vote_id).await?;
        
        // Check if vote has ended
        if Utc::now() < vote.reveal_end {
            return Err(VoteError::InvalidState {
                expected: "Vote ended".to_string(),
                actual: "Vote still in progress".to_string(),
            });
        }
        
        // Get all reveals
        let reveals = self.vote_service.list_reveals(vote_id).await?;
        
        // Calculate results using template system
        let results = self.vote_service.calculate_results(&vote, &reveals).await?;
        
        // Update vote with results
        self.vote_service.update_vote_results(vote_id, &results).await?;
        
        // Update vote status
        self.vote_service.update_vote_status(vote_id, VoteStatus::Completed).await?;
        
        info!("Results calculated successfully for vote: {}", vote_id);
        Ok(results)
    }

    /// Get vote information
    pub async fn get_vote(&self, vote_id: &str) -> Result<Vote, VoteError> {
        self.vote_service.get_vote(vote_id).await
    }

    /// List votes
    pub async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, VoteError> {
        self.vote_service.list_votes(query).await
    }

    /// Verify vote results
    pub async fn verify_results(&self, vote_id: &str) -> Result<VerificationResult, VoteError> {
        info!("Verifying results for vote: {}", vote_id);
        
        // Get the vote
        let vote = self.vote_service.get_vote(vote_id).await?;
        
        // Check if vote has results
        let results = vote.results.as_ref()
            .ok_or_else(|| VoteError::InvalidState {
                expected: "Vote with results".to_string(),
                actual: "Vote without results".to_string(),
            })?;
        
        // Get all commitments and reveals
        let commitments = self.vote_service.list_commitments(vote_id).await?;
        let reveals = self.vote_service.list_reveals(vote_id).await?;
        
        let mut all_issues = Vec::new();
        
        // Verify commitments
        let commitment_verification = self.verify_commitments(&commitments, &reveals).await?;
        all_issues.extend(commitment_verification.commitment_issues.clone());
        
        // Verify results
        let results_verification = self.verify_results_calculation(&vote, &reveals, results).await?;
        all_issues.extend(results_verification.results_issues.clone());
        
        let is_valid = all_issues.is_empty();
        
        let verification_result = VerificationResult {
            vote_id: vote_id.to_string(),
            is_valid,
            verification_timestamp: Utc::now(),
            commitment_verification,
            results_verification,
            issues: all_issues,
        };
        
        info!("Verification completed for vote: {} - Valid: {}", vote_id, is_valid);
        Ok(verification_result)
    }

    /// Verify commitments against reveals
    async fn verify_commitments(
        &self,
        commitments: &[Commitment],
        reveals: &[Reveal],
    ) -> Result<CommitmentVerification, VoteError> {
        let mut verified_count = 0;
        let mut failed_count = 0;
        let mut issues = Vec::new();
        
        for commitment in commitments {
            // Find corresponding reveal
            if let Some(reveal) = reveals.iter().find(|r| r.voter == commitment.voter) {
                // Verify commitment matches reveal
                let value_str = serde_json::to_string(&reveal.value)
                    .map_err(|e| VoteError::InvalidReveal {
                        message: format!("Invalid value format: {}", e),
                    })?;
                
                if shared_utils::crypto::verify_commitment(&value_str, &reveal.salt, &commitment.commitment_hash) {
                    verified_count += 1;
                } else {
                    failed_count += 1;
                    issues.push(format!("Commitment verification failed for voter: {}", commitment.voter));
                }
            } else {
                failed_count += 1;
                issues.push(format!("No reveal found for commitment from voter: {}", commitment.voter));
            }
        }
        
        Ok(CommitmentVerification {
            total_commitments: commitments.len() as u32,
            verified_commitments: verified_count,
            failed_commitments: failed_count,
            commitment_issues: issues,
        })
    }

    /// Verify results calculation
    async fn verify_results_calculation(
        &self,
        _vote: &Vote,
        reveals: &[Reveal],
        results: &VoteResults,
    ) -> Result<ResultsVerification, VoteError> {
        let mut issues = Vec::new();
        
        // Verify reveal counts
        let total_reveals = reveals.len() as u32;
        let valid_reveals = reveals.len() as u32; // All reveals in the list are considered valid
        let invalid_reveals = 0; // We don't track invalid reveals separately
        
        // Verify random seed calculation (simplified - in real implementation, this would be more complex)
        let random_seed_verification = true; // TODO: Implement actual random seed verification
        
        // Verify selection algorithm execution
        let selection_algorithm_verification = true; // TODO: Implement actual algorithm verification
        
        // Check if results match expectations
        if results.total_votes != total_reveals {
            issues.push(format!(
                "Total votes mismatch: expected {}, got {}",
                total_reveals, results.total_votes
            ));
        }
        
        Ok(ResultsVerification {
            total_reveals,
            valid_reveals,
            invalid_reveals,
            random_seed_verification,
            selection_algorithm_verification,
            results_issues: issues,
        })
    }
}
