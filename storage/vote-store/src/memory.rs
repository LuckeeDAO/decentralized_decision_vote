use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use shared_types::*;
use tracing::debug;

use crate::traits::{VoteStore, StoreError, StoreStats};

/// In-memory implementation of VoteStore
pub struct MemoryVoteStore {
    votes: Arc<RwLock<HashMap<String, Vote>>>,
    commitments: Arc<RwLock<HashMap<String, Commitment>>>,
    reveals: Arc<RwLock<HashMap<String, Reveal>>>,
}

impl Default for MemoryVoteStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryVoteStore {
    pub fn new() -> Self {
        Self {
            votes: Arc::new(RwLock::new(HashMap::new())),
            commitments: Arc::new(RwLock::new(HashMap::new())),
            reveals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl VoteStore for MemoryVoteStore {
    async fn create_vote(&self, vote: Vote) -> Result<(), StoreError> {
        debug!("Creating vote: {}", vote.id);
        let mut votes = self.votes.write().await;
        votes.insert(vote.id.clone(), vote);
        Ok(())
    }

    async fn get_vote(&self, id: &str) -> Result<Vote, StoreError> {
        debug!("Getting vote: {}", id);
        let votes = self.votes.read().await;
        votes.get(id)
            .cloned()
            .ok_or_else(|| StoreError::VoteNotFound { id: id.to_string() })
    }

    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, StoreError> {
        debug!("Listing votes: page={}, size={}", query.page, query.page_size);
        let votes = self.votes.read().await;
        
        let mut all_votes: Vec<Vote> = votes.values().cloned().collect();
        
        // Apply filters
        if let Some(status) = &query.status {
            all_votes.retain(|v| std::mem::discriminant(&v.status) == std::mem::discriminant(status));
        }
        
        if let Some(creator) = &query.creator {
            all_votes.retain(|v| v.creator == *creator);
        }
        
        // Sort by creation time (newest first)
        all_votes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Apply pagination
        let start = (query.page * query.page_size) as usize;
        let _end = start + query.page_size as usize;
        
        let items = all_votes.into_iter()
            .skip(start)
            .take(query.page_size as usize)
            .collect();
        
        let total = votes.len() as u32;
        let total_pages = total.div_ceil(query.page_size);
        
        Ok(Page {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
            total_pages,
        })
    }

    async fn update_vote_status(&self, id: &str, status: VoteStatus) -> Result<(), StoreError> {
        debug!("Updating vote status: {} -> {:?}", id, status);
        let mut votes = self.votes.write().await;
        if let Some(vote) = votes.get_mut(id) {
            vote.status = status;
            Ok(())
        } else {
            Err(StoreError::VoteNotFound { id: id.to_string() })
        }
    }

    async fn update_vote_results(&self, id: &str, results: &VoteResults) -> Result<(), StoreError> {
        debug!("Updating vote results: {}", id);
        let mut votes = self.votes.write().await;
        if let Some(vote) = votes.get_mut(id) {
            vote.results = Some(results.clone());
            Ok(())
        } else {
            Err(StoreError::VoteNotFound { id: id.to_string() })
        }
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), StoreError> {
        debug!("Saving commitment: {}", commitment.id);
        let mut commitments = self.commitments.write().await;
        commitments.insert(commitment.id.clone(), commitment);
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, StoreError> {
        debug!("Getting commitment: {}:{}", vote_id, voter);
        let commitments = self.commitments.read().await;
        let commitment = commitments.values()
            .find(|c| c.vote_id == vote_id && c.voter == voter)
            .cloned();
        Ok(commitment)
    }

    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, StoreError> {
        debug!("Listing commitments for vote: {}", vote_id);
        let commitments = self.commitments.read().await;
        let vote_commitments: Vec<Commitment> = commitments.values()
            .filter(|c| c.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(vote_commitments)
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), StoreError> {
        debug!("Saving reveal: {}", reveal.id);
        let mut reveals = self.reveals.write().await;
        reveals.insert(reveal.id.clone(), reveal);
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, StoreError> {
        debug!("Listing reveals for vote: {}", vote_id);
        let reveals = self.reveals.read().await;
        let vote_reveals: Vec<Reveal> = reveals.values()
            .filter(|r| r.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(vote_reveals)
    }

    async fn get_reveal(&self, vote_id: &str, voter: &str) -> Result<Option<Reveal>, StoreError> {
        debug!("Getting reveal: {}:{}", vote_id, voter);
        let reveals = self.reveals.read().await;
        let reveal = reveals.values()
            .find(|r| r.vote_id == vote_id && r.voter == voter)
            .cloned();
        Ok(reveal)
    }

    async fn delete_vote(&self, id: &str) -> Result<(), StoreError> {
        debug!("Deleting vote: {}", id);
        let mut votes = self.votes.write().await;
        votes.remove(id);
        
        // Also remove related commitments and reveals
        let mut commitments = self.commitments.write().await;
        commitments.retain(|_, c| c.vote_id != id);
        
        let mut reveals = self.reveals.write().await;
        reveals.retain(|_, r| r.vote_id != id);
        
        Ok(())
    }

    async fn get_stats(&self) -> Result<StoreStats, StoreError> {
        debug!("Getting storage stats");
        let votes = self.votes.read().await;
        let commitments = self.commitments.read().await;
        let reveals = self.reveals.read().await;
        
        let total_votes = votes.len() as u32;
        let total_commitments = commitments.len() as u32;
        let total_reveals = reveals.len() as u32;
        
        let active_votes = votes.values()
            .filter(|v| matches!(v.status, VoteStatus::Created | VoteStatus::CommitmentPhase | VoteStatus::RevealPhase))
            .count() as u32;
        
        let completed_votes = votes.values()
            .filter(|v| matches!(v.status, VoteStatus::Completed))
            .count() as u32;
        
        Ok(StoreStats {
            total_votes,
            total_commitments,
            total_reveals,
            active_votes,
            completed_votes,
        })
    }
}
