use std::sync::{Arc, Mutex};
use chrono::Utc;
use shared_types::*;
use shared_utils::crypto::create_commitment;
use vote_engine::*;

// Mock implementations for testing
struct MockVoteService {
    votes: Mutex<std::collections::HashMap<String, Vote>>,
    commitments: Mutex<std::collections::HashMap<String, Commitment>>,
    reveals: Mutex<std::collections::HashMap<String, Reveal>>,
}

impl MockVoteService {
    fn new() -> Self {
        Self {
            votes: Mutex::new(std::collections::HashMap::new()),
            commitments: Mutex::new(std::collections::HashMap::new()),
            reveals: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VoteService for MockVoteService {
    async fn create_vote(&self, vote: Vote) -> Result<(), VoteError> {
        // Store the vote in our mock HashMap
        let mut votes = self.votes.lock().unwrap();
        votes.insert(vote.id.clone(), vote);
        Ok(())
    }

    async fn get_vote(&self, vote_id: &str) -> Result<Vote, VoteError> {
        let votes = self.votes.lock().unwrap();
        votes.get(vote_id)
            .cloned()
            .ok_or_else(|| VoteError::VoteNotFound {
                id: vote_id.to_string(),
            })
    }

    async fn list_votes(&self, _query: ListQuery) -> Result<Page<Vote>, VoteError> {
        let votes_guard = self.votes.lock().unwrap();
        let votes: Vec<Vote> = votes_guard.values().cloned().collect();
        let total = votes_guard.len() as u32;
        Ok(Page {
            items: votes,
            total,
            page: 1,
            page_size: 10,
            total_pages: 1,
        })
    }

    async fn update_vote_status(&self, vote_id: &str, status: VoteStatus) -> Result<(), VoteError> {
        let mut votes = self.votes.lock().unwrap();
        if let Some(vote) = votes.get_mut(vote_id) {
            vote.status = status;
        }
        Ok(())
    }

    async fn update_vote_results(&self, _vote_id: &str, _results: &VoteResults) -> Result<(), VoteError> {
        Ok(())
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), VoteError> {
        // Store the commitment in our mock HashMap
        let key = format!("{}:{}", commitment.vote_id, commitment.voter);
        let mut commitments = self.commitments.lock().unwrap();
        commitments.insert(key, commitment);
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, VoteError> {
        let key = format!("{}:{}", vote_id, voter);
        let commitments = self.commitments.lock().unwrap();
        Ok(commitments.get(&key).cloned())
    }

    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, VoteError> {
        let commitments_guard = self.commitments.lock().unwrap();
        let commitments: Vec<Commitment> = commitments_guard
            .values()
            .filter(|c| c.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(commitments)
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), VoteError> {
        // Store the reveal in our mock HashMap
        let key = format!("{}:{}", reveal.vote_id, reveal.voter);
        let mut reveals = self.reveals.lock().unwrap();
        reveals.insert(key, reveal);
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, VoteError> {
        let reveals_guard = self.reveals.lock().unwrap();
        let reveals: Vec<Reveal> = reveals_guard
            .values()
            .filter(|r| r.vote_id == vote_id)
            .cloned()
            .collect();
        Ok(reveals)
    }

    async fn calculate_results(&self, _vote: &Vote, _reveals: &[Reveal]) -> Result<VoteResults, VoteError> {
        Ok(VoteResults {
            vote_id: "test".to_string(),
            total_votes: 0,
            results: serde_json::Value::Object(serde_json::Map::new()),
            calculated_at: Utc::now(),
        })
    }
}

#[tokio::test]
async fn test_create_vote_success() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "simple".to_string(),
        template_params: serde_json::Value::Object(serde_json::Map::new()),
        commitment_duration_hours: 24,
        reveal_duration_hours: 24,
    };

    let result = engine.create_vote(config).await;
    assert!(result.is_ok());
    
    let vote_id = result.unwrap();
    assert!(!vote_id.is_empty());
}

#[tokio::test]
async fn test_create_vote_invalid_config() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    let config = VoteConfig {
        title: "".to_string(), // Invalid: empty title
        description: "A test vote".to_string(),
        template_id: "simple".to_string(),
        template_params: serde_json::Value::Object(serde_json::Map::new()),
        commitment_duration_hours: 24,
        reveal_duration_hours: 24,
    };

    let result = engine.create_vote(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_commit_vote_success() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    // First create a vote
    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "simple".to_string(),
        template_params: serde_json::Value::Object(serde_json::Map::new()),
        commitment_duration_hours: 24,
        reveal_duration_hours: 24,
    };

    let vote_id = engine.create_vote(config).await.unwrap();

    // Now commit a vote
    let commit_request = CommitRequest {
        voter: "test_voter".to_string(),
        commitment_hash: "a".repeat(64), // Valid hash length
        salt: "test_salt".to_string(),
    };

    let result = engine.commit_vote(&vote_id, commit_request).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.success);
    assert!(!response.commitment_id.is_empty());
}

#[tokio::test]
async fn test_commit_vote_invalid_vote_id() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    let commit_request = CommitRequest {
        voter: "test_voter".to_string(),
        commitment_hash: "a".repeat(64), // Valid hash length
        salt: "test_salt".to_string(),
    };

    let result = engine.commit_vote("nonexistent_vote", commit_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reveal_vote_success() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service.clone());

    // First create a vote
    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "simple".to_string(),
        template_params: serde_json::Value::Object(serde_json::Map::new()),
        commitment_duration_hours: 1, // Keep 1 hour for now
        reveal_duration_hours: 1,
    };

    let vote_id = engine.create_vote(config).await.unwrap();

    // First commit a vote
    let salt = "test_salt".to_string();
    let value = serde_json::Value::String("yes".to_string());
    let value_str = serde_json::to_string(&value).unwrap();
    let commitment_hash = create_commitment(&value_str, &salt);
    
    let commit_request = CommitRequest {
        voter: "test_voter".to_string(),
        commitment_hash,
        salt: salt.clone(),
    };

    let commit_result = engine.commit_vote(&vote_id, commit_request).await;
    assert!(commit_result.is_ok());

    // Manually advance the vote to reveal phase for testing
    // In a real scenario, this would happen automatically when the commitment phase ends
    let vote = mock_service.get_vote(&vote_id).await.unwrap();
    let mut updated_vote = vote.clone();
    updated_vote.status = VoteStatus::RevealPhase;
    updated_vote.reveal_start = chrono::Utc::now() - chrono::Duration::hours(1); // Set reveal start to 1 hour ago
    mock_service.create_vote(updated_vote).await.unwrap();

    // Now reveal a vote
    let reveal_request = RevealRequest {
        voter: "test_voter".to_string(),
        value,
        salt,
    };

    let result = engine.reveal_vote(&vote_id, reveal_request).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.success);
    assert!(!response.reveal_id.is_empty());
}

#[tokio::test]
async fn test_get_results_vote_not_ended() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    // Create a vote with long duration
    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "simple".to_string(),
        template_params: serde_json::Value::Object(serde_json::Map::new()),
        commitment_duration_hours: 24,
        reveal_duration_hours: 24,
    };

    let vote_id = engine.create_vote(config).await.unwrap();

    // Try to get results before vote ends
    let result = engine.get_results(&vote_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_votes() {
    let mock_service = Arc::new(MockVoteService::new());
    let engine = VoteEngine::new(mock_service);

    let query = ListQuery {
        page: 1,
        page_size: 10,
        status: None,
        creator: None,
    };

    let result = engine.list_votes(query).await;
    assert!(result.is_ok());
    
    let page = result.unwrap();
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 10);
}
