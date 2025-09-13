use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: String,
    pub title: String,
    pub description: String,
    pub template_id: String,
    pub template_params: serde_json::Value,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub commitment_start: DateTime<Utc>,
    pub commitment_end: DateTime<Utc>,
    pub reveal_start: DateTime<Utc>,
    pub reveal_end: DateTime<Utc>,
    pub status: VoteStatus,
    pub results: Option<VoteResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteStatus {
    Created,
    CommitmentPhase,
    RevealPhase,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteConfig {
    pub title: String,
    pub description: String,
    pub template_id: String,
    pub template_params: serde_json::Value,
    pub commitment_duration_hours: u32,
    pub reveal_duration_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: String,
    pub vote_id: String,
    pub voter: String,
    pub commitment_hash: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reveal {
    pub id: String,
    pub vote_id: String,
    pub voter: String,
    pub value: serde_json::Value,
    pub salt: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResults {
    pub vote_id: String,
    pub total_votes: u32,
    pub results: serde_json::Value,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub voter: String,
    pub commitment_hash: String,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResponse {
    pub commitment_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealRequest {
    pub voter: String,
    pub value: serde_json::Value,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealResponse {
    pub reveal_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    pub page: u32,
    pub page_size: u32,
    pub status: Option<VoteStatus>,
    pub creator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub vote_id: String,
    pub is_valid: bool,
    pub verification_timestamp: DateTime<Utc>,
    pub commitment_verification: CommitmentVerification,
    pub results_verification: ResultsVerification,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentVerification {
    pub total_commitments: u32,
    pub verified_commitments: u32,
    pub failed_commitments: u32,
    pub commitment_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsVerification {
    pub total_reveals: u32,
    pub valid_reveals: u32,
    pub invalid_reveals: u32,
    pub random_seed_verification: bool,
    pub selection_algorithm_verification: bool,
    pub results_issues: Vec<String>,
}
