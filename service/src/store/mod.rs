pub mod memory;
use async_trait::async_trait;
use crate::model::vote::*;

#[derive(thiserror::Error, Debug)]
pub enum StoreError { #[error("not found")] NotFound, #[error("conflict")] Conflict, #[error("io")] Io, #[error("internal")] Internal }

#[async_trait]
pub trait VoteStore: Send + Sync {
    async fn create_vote(&self, cfg: VoteConfig) -> Result<String, StoreError>;
    async fn get_vote(&self, id: &str) -> Result<VoteDetailDto, StoreError>;
    async fn list_votes(&self, offset: u64, limit: u64) -> Result<(Vec<VoteSummaryDto>, u64), StoreError>;
    async fn put_commitment(&self, vote_id: &str, commitment: Commitment) -> Result<(), StoreError>;
    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, StoreError>;
    async fn put_reveal(&self, vote_id: &str, reveal: Reveal) -> Result<(), StoreError>;
    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, StoreError>;
}

