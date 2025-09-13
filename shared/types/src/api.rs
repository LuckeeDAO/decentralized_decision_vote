use serde::{Deserialize, Serialize};
use crate::vote::*;

// API Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVoteRequest {
    pub config: VoteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVoteResponse {
    pub vote_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVoteResponse {
    pub vote: Vote,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVotesResponse {
    pub votes: Page<Vote>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResultsResponse {
    pub results: VoteResults,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResultsResponse {
    pub verification: VerificationResult,
    pub success: bool,
}

// WebSocket message types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: MessageType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    VoteCreated,
    VoteUpdated,
    CommitmentReceived,
    RevealReceived,
    ResultsCalculated,
    Error,
}

// Health check types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub services: std::collections::HashMap<String, ServiceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: String,
    pub message: Option<String>,
}
