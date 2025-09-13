use async_trait::async_trait;
use std::sync::Arc;
use shared_types::*;

/// Service trait for vote operations
#[async_trait]
pub trait VoteService: Send + Sync {
    async fn create_vote(&self, vote: Vote) -> Result<(), VoteError>;
    async fn get_vote(&self, id: &str) -> Result<Vote, VoteError>;
    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, VoteError>;
    async fn update_vote_status(&self, id: &str, status: VoteStatus) -> Result<(), VoteError>;
    async fn update_vote_results(&self, id: &str, results: &VoteResults) -> Result<(), VoteError>;
    
    async fn save_commitment(&self, commitment: Commitment) -> Result<(), VoteError>;
    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, VoteError>;
    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, VoteError>;
    
    async fn save_reveal(&self, reveal: Reveal) -> Result<(), VoteError>;
    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, VoteError>;
    
    async fn calculate_results(&self, vote: &Vote, reveals: &[Reveal]) -> Result<VoteResults, VoteError>;
}

/// In-memory implementation of VoteService for testing
pub struct MemoryVoteService {
    votes: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vote>>>,
    commitments: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Commitment>>>,
    reveals: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Reveal>>>,
}

impl Default for MemoryVoteService {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryVoteService {
    pub fn new() -> Self {
        Self {
            votes: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            commitments: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            reveals: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl VoteService for MemoryVoteService {
    async fn create_vote(&self, vote: Vote) -> Result<(), VoteError> {
        let mut votes = self.votes.write().await;
        votes.insert(vote.id.clone(), vote);
        Ok(())
    }

    async fn get_vote(&self, id: &str) -> Result<Vote, VoteError> {
        let votes = self.votes.read().await;
        votes.get(id)
            .cloned()
            .ok_or_else(|| VoteError::VoteNotFound { id: id.to_string() })
    }

    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, VoteError> {
        let votes = self.votes.read().await;
        let all_votes: Vec<Vote> = votes.values().cloned().collect();
        
        let start = (query.page * query.page_size) as usize;
        
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

    async fn update_vote_status(&self, id: &str, status: VoteStatus) -> Result<(), VoteError> {
        let mut votes = self.votes.write().await;
        if let Some(vote) = votes.get_mut(id) {
            vote.status = status;
            Ok(())
        } else {
            Err(VoteError::VoteNotFound { id: id.to_string() })
        }
    }

    async fn update_vote_results(&self, id: &str, results: &VoteResults) -> Result<(), VoteError> {
        let mut votes = self.votes.write().await;
        if let Some(vote) = votes.get_mut(id) {
            vote.results = Some(results.clone());
            Ok(())
        } else {
            Err(VoteError::VoteNotFound { id: id.to_string() })
        }
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), VoteError> {
        let mut commitments = self.commitments.write().await;
        commitments.insert(commitment.id.clone(), commitment);
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, VoteError> {
        let commitments = self.commitments.read().await;
        let commitment = commitments.values()
            .find(|c| c.vote_id == vote_id && c.voter == voter)
            .cloned();
        Ok(commitment)
    }

    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, VoteError> {
        let commitments = self.commitments.read().await;
        let vote_commitments: Vec<Commitment> = commitments.values()
            .filter(|c| c.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(vote_commitments)
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), VoteError> {
        let mut reveals = self.reveals.write().await;
        reveals.insert(reveal.id.clone(), reveal);
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, VoteError> {
        let reveals = self.reveals.read().await;
        let vote_reveals: Vec<Reveal> = reveals.values()
            .filter(|r| r.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(vote_reveals)
    }

    async fn calculate_results(&self, vote: &Vote, reveals: &[Reveal]) -> Result<VoteResults, VoteError> {
        // Simple aggregation for now - in real implementation, this would use the template system
        let total_votes = reveals.len() as u32;
        
        // Create a simple results structure
        let mut results_map = std::collections::HashMap::new();
        for reveal in reveals {
            let value_str = serde_json::to_string(&reveal.value)
                .unwrap_or_else(|_| "unknown".to_string());
            *results_map.entry(value_str).or_insert(0) += 1;
        }
        
        let results = VoteResults {
            vote_id: vote.id.clone(),
            total_votes,
            results: serde_json::to_value(results_map)
                .map_err(VoteError::SerializationError)?,
            calculated_at: chrono::Utc::now(),
        };
        
        Ok(results)
    }
}
