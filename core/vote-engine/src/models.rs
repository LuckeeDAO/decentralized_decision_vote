// Additional models specific to the vote engine
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Vote statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteStats {
    pub vote_id: String,
    pub total_commitments: u32,
    pub total_reveals: u32,
    pub commitment_rate: f64,
    pub reveal_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Vote phase information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotePhase {
    pub vote_id: String,
    pub current_phase: String,
    pub phase_start: DateTime<Utc>,
    pub phase_end: DateTime<Utc>,
    pub time_remaining_seconds: i64,
    pub progress_percentage: f64,
}

/// Vote audit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteAudit {
    pub vote_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub last_modified: DateTime<Utc>,
    pub modification_count: u32,
    pub events: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub user: Option<String>,
    pub metadata: serde_json::Value,
}
