use std::sync::Arc;
use shared_types::*;
use vote_engine::*;
use storage_vote_store::*;

/// Test environment setup for integration tests
pub struct TestEnvironment {
    pub vote_engine: Arc<VoteEngine>,
    pub vote_service: Arc<dyn VoteService>,
    pub storage: Arc<dyn VoteStore>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        // Create in-memory storage for testing
        let storage = Arc::new(MemoryVoteStore::new());
        
        // Create vote service
        let vote_service = Arc::new(TestVoteService::new(storage.clone()));
        
        // Create vote engine
        let vote_engine = Arc::new(VoteEngine::new(vote_service.clone()));
        
        Self {
            vote_engine,
            vote_service,
            storage,
        }
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data
        // In a real implementation, this would clear the database
    }
}

// Mock vote service for testing
pub struct TestVoteService {
    storage: Arc<dyn VoteStore>,
}

impl TestVoteService {
    pub fn new(storage: Arc<dyn VoteStore>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl VoteService for TestVoteService {
    async fn create_vote(&self, vote: Vote) -> Result<(), VoteError> {
        self.storage.save_vote(vote).await
    }

    async fn get_vote(&self, vote_id: &str) -> Result<Vote, VoteError> {
        self.storage.get_vote(vote_id).await
    }

    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, VoteError> {
        self.storage.list_votes(query).await
    }

    async fn update_vote_status(&self, vote_id: &str, status: VoteStatus) -> Result<(), VoteError> {
        let mut vote = self.storage.get_vote(vote_id).await?;
        vote.status = status;
        self.storage.save_vote(vote).await
    }

    async fn update_vote_results(&self, vote_id: &str, results: &VoteResults) -> Result<(), VoteError> {
        let mut vote = self.storage.get_vote(vote_id).await?;
        vote.results = Some(results.clone());
        self.storage.save_vote(vote).await
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), VoteError> {
        self.storage.save_commitment(commitment).await
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, VoteError> {
        self.storage.get_commitment(vote_id, voter).await
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), VoteError> {
        self.storage.save_reveal(reveal).await
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, VoteError> {
        self.storage.list_reveals(vote_id).await
    }

    async fn calculate_results(&self, vote: &Vote, reveals: &[Reveal]) -> Result<VoteResults, VoteError> {
        // Simple result calculation for testing
        let mut results = std::collections::HashMap::new();
        
        for reveal in reveals {
            let value = reveal.value.as_str().unwrap_or("unknown");
            *results.entry(value.to_string()).or_insert(0) += 1;
        }
        
        Ok(VoteResults {
            vote_id: vote.id.clone(),
            total_votes: reveals.len() as u32,
            results: serde_json::to_value(results).unwrap(),
            calculated_at: chrono::Utc::now(),
        })
    }
}

// Mock storage implementation for testing
pub struct MemoryVoteStore {
    votes: std::sync::RwLock<std::collections::HashMap<String, Vote>>,
    commitments: std::sync::RwLock<std::collections::HashMap<String, Commitment>>,
    reveals: std::sync::RwLock<std::collections::HashMap<String, Reveal>>,
}

impl MemoryVoteStore {
    pub fn new() -> Self {
        Self {
            votes: std::sync::RwLock::new(std::collections::HashMap::new()),
            commitments: std::sync::RwLock::new(std::collections::HashMap::new()),
            reveals: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VoteStore for MemoryVoteStore {
    async fn save_vote(&self, vote: Vote) -> Result<(), VoteError> {
        let mut votes = self.votes.write().unwrap();
        votes.insert(vote.id.clone(), vote);
        Ok(())
    }

    async fn get_vote(&self, vote_id: &str) -> Result<Vote, VoteError> {
        let votes = self.votes.read().unwrap();
        votes.get(vote_id)
            .cloned()
            .ok_or_else(|| VoteError::NotFound {
                resource: "vote".to_string(),
                id: vote_id.to_string(),
            })
    }

    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, VoteError> {
        let votes = self.votes.read().unwrap();
        let all_votes: Vec<Vote> = votes.values().cloned().collect();
        
        let start = (query.page - 1) * query.page_size;
        let end = start + query.page_size;
        
        let items = all_votes.into_iter()
            .skip(start as usize)
            .take(query.page_size as usize)
            .collect();
        
        Ok(Page {
            items,
            total: votes.len() as u32,
            page: query.page,
            page_size: query.page_size,
            total_pages: (votes.len() as u32 + query.page_size - 1) / query.page_size,
        })
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), VoteError> {
        let mut commitments = self.commitments.write().unwrap();
        commitments.insert(commitment.id.clone(), commitment);
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, VoteError> {
        let commitments = self.commitments.read().unwrap();
        Ok(commitments.values()
            .find(|c| c.vote_id == vote_id && c.voter == voter)
            .cloned())
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), VoteError> {
        let mut reveals = self.reveals.write().unwrap();
        reveals.insert(reveal.id.clone(), reveal);
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, VoteError> {
        let reveals = self.reveals.read().unwrap();
        Ok(reveals.values()
            .filter(|r| r.vote_id == vote_id)
            .cloned()
            .collect())
    }
}
