use async_trait::async_trait;
use shared_types::*;

/// Trait for vote storage operations
#[async_trait]
pub trait VoteStore: Send + Sync {
    /// Create a new vote
    async fn create_vote(&self, vote: Vote) -> Result<(), StoreError>;
    
    /// Get a vote by ID
    async fn get_vote(&self, id: &str) -> Result<Vote, StoreError>;
    
    /// List votes with pagination
    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, StoreError>;
    
    /// Update vote status
    async fn update_vote_status(&self, id: &str, status: VoteStatus) -> Result<(), StoreError>;
    
    /// Update vote results
    async fn update_vote_results(&self, id: &str, results: &VoteResults) -> Result<(), StoreError>;
    
    /// Save a commitment
    async fn save_commitment(&self, commitment: Commitment) -> Result<(), StoreError>;
    
    /// Get a commitment by vote ID and voter
    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, StoreError>;
    
    /// List commitments for a vote
    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, StoreError>;
    
    /// Save a reveal
    async fn save_reveal(&self, reveal: Reveal) -> Result<(), StoreError>;
    
    /// List reveals for a vote
    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, StoreError>;
    
    /// Get reveal by vote ID and voter
    async fn get_reveal(&self, vote_id: &str, voter: &str) -> Result<Option<Reveal>, StoreError>;
    
    /// Delete a vote (for cleanup)
    async fn delete_vote(&self, id: &str) -> Result<(), StoreError>;
    
    /// Get storage statistics
    async fn get_stats(&self) -> Result<StoreStats, StoreError>;
}

/// Storage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoreStats {
    pub total_votes: u32,
    pub total_commitments: u32,
    pub total_reveals: u32,
    pub active_votes: u32,
    pub completed_votes: u32,
}

/// Storage errors
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("Vote not found: {id}")]
    VoteNotFound { id: String },
    
    #[error("Commitment not found: {vote_id}:{voter}")]
    CommitmentNotFound { vote_id: String, voter: String },
    
    #[error("Reveal not found: {vote_id}:{voter}")]
    RevealNotFound { vote_id: String, voter: String },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(#[from] chrono::format::ParseError),
}
