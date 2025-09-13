use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use crate::model::vote::*;
use super::{VoteStore, StoreError};
use async_trait::async_trait;

#[derive(Default)]
pub struct MemoryVoteStore {
    inner: Arc<RwLock<MemoryDb>>,    
}

#[derive(Default)]
struct MemoryDb {
    votes: HashMap<String, (VoteConfig, i64)>,
    commitments: HashMap<(String, String), Commitment>,
    reveals: HashMap<(String, String), Reveal>,
}

#[async_trait]
impl VoteStore for MemoryVoteStore {
    async fn create_vote(&self, cfg: VoteConfig) -> Result<String, StoreError> {
        let mut g = self.inner.write().await;
        let id = Uuid::new_v4().to_string();
        let created_ts = Utc::now().timestamp();
        g.votes.insert(id.clone(), (cfg, created_ts));
        Ok(id)
    }

    async fn get_vote(&self, id: &str) -> Result<VoteDetailDto, StoreError> {
        let g = self.inner.read().await;
        let (cfg, created_ts) = g.votes.get(id).ok_or(StoreError::NotFound)?.clone();
        let num_commitments = g.commitments.keys().filter(|(vid, _)| vid == id).count() as u64;
        let num_reveals = g.reveals.keys().filter(|(vid, _)| vid == id).count() as u64;
        Ok(VoteDetailDto { id: id.to_string(), config: cfg, created_ts, num_commitments, num_reveals })
    }

    async fn list_votes(&self, offset: u64, limit: u64) -> Result<(Vec<VoteSummaryDto>, u64), StoreError> {
        let g = self.inner.read().await;
        let mut items: Vec<(String, (VoteConfig, i64))> = g.votes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        items.sort_by_key(|(_, (_, ts))| *ts);
        let total = items.len() as u64;
        let slice = items.into_iter().skip(offset as usize).take(limit as usize);
        let summaries = slice.map(|(id, (cfg, _))| {
            let status = "draft".to_string();
            VoteSummaryDto { id, title: cfg.title.clone(), commit_window: (cfg.commit_start_height, cfg.commit_end_height), reveal_window: (cfg.reveal_start_height, cfg.reveal_end_height), status }
        }).collect();
        Ok((summaries, total))
    }

    async fn put_commitment(&self, vote_id: &str, commitment: Commitment) -> Result<(), StoreError> {
        let mut g = self.inner.write().await;
        let key = (vote_id.to_string(), commitment.voter.clone());
        if g.commitments.contains_key(&key) { return Err(StoreError::Conflict); }
        g.commitments.insert(key, commitment);
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, StoreError> {
        let g = self.inner.read().await;
        Ok(g.commitments.get(&(vote_id.to_string(), voter.to_string())).cloned())
    }

    async fn put_reveal(&self, vote_id: &str, reveal: Reveal) -> Result<(), StoreError> {
        let mut g = self.inner.write().await;
        let key = (vote_id.to_string(), reveal.voter.clone());
        if g.reveals.contains_key(&key) { return Err(StoreError::Conflict); }
        g.reveals.insert(key, reveal);
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, StoreError> {
        let g = self.inner.read().await;
        Ok(g.reveals.iter().filter(|((vid, _), _)| vid == vote_id).map(|(_, v)| v.clone()).collect())
    }
}

impl MemoryVoteStore {
    pub fn new() -> Self { Self::default() }
}


