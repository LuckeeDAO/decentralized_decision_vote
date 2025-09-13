use decentralized_decision_vote::service::{VoteService, VoteServiceImpl};
use decentralized_decision_vote::store::memory::MemoryVoteStore;
use decentralized_decision_vote::core::template::{TemplateRegistry, BitTemplate, OptionIndexTemplate, StringTemplate};
use decentralized_decision_vote::model::vote::*;
use serde_json::json;
use std::sync::Arc;

async fn create_test_service() -> VoteServiceImpl {
    let store: Arc<dyn decentralized_decision_vote::store::VoteStore> = Arc::new(MemoryVoteStore::default());
    let mut registry = TemplateRegistry::new();
    registry.register(BitTemplate);
    registry.register(OptionIndexTemplate);
    registry.register(StringTemplate);
    VoteServiceImpl::new(store, Arc::new(registry))
}

#[tokio::test]
async fn test_vote_lifecycle() {
    let service = create_test_service().await;
    
    // Create vote
    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: Some("Test description".to_string()),
        options: vec!["Yes".to_string(), "No".to_string()],
        commit_start_height: 0,
        commit_end_height: 100,
        reveal_start_height: 101,
        reveal_end_height: 200,
        participants: vec!["alice".to_string(), "bob".to_string()],
        value_template: "option_index".to_string(),
        template_params: json!({"max": 2}),
    };
    
    let vote_id = service.create_vote(config).await.unwrap();
    assert!(!vote_id.is_empty());
    
    // Get vote
    let vote = service.get_vote(&vote_id).await.unwrap();
    assert_eq!(vote.config.title, "Test Vote");
    assert_eq!(vote.config.options.len(), 2);
    
    // Commit vote
    let commit_resp = service.commit(&vote_id, "alice", json!(0), "deadbeef".to_string()).await.unwrap();
    assert!(!commit_resp.commitment_hex.is_empty());
    
    // Reveal vote
    let reveal_resp = service.reveal(&vote_id, "alice", json!(0), "deadbeef".to_string()).await.unwrap();
    assert!(reveal_resp.accepted);
    
    // Get results
    let results = service.results(&vote_id).await.unwrap();
    assert_eq!(results.vote_id, vote_id);
    assert_eq!(results.result, json!(1)); // count of reveals
}

#[tokio::test]
async fn test_participant_whitelist() {
    let service = create_test_service().await;
    
    let config = VoteConfig {
        title: "Whitelist Test".to_string(),
        description: None,
        options: vec!["Option 1".to_string()],
        commit_start_height: 0,
        commit_end_height: 100,
        reveal_start_height: 101,
        reveal_end_height: 200,
        participants: vec!["alice".to_string()], // Only alice allowed
        value_template: "bit".to_string(),
        template_params: json!({}),
    };
    
    let vote_id = service.create_vote(config).await.unwrap();
    
    // Alice should be able to commit
    let result = service.commit(&vote_id, "alice", json!(true), "salt1".to_string()).await;
    assert!(result.is_ok());
    
    // Bob should be rejected
    let result = service.commit(&vote_id, "bob", json!(true), "salt2".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_idempotent_commit_reveal() {
    let service = create_test_service().await;
    
    let config = VoteConfig {
        title: "Idempotent Test".to_string(),
        description: None,
        options: vec!["Option 1".to_string()],
        commit_start_height: 0,
        commit_end_height: 100,
        reveal_start_height: 101,
        reveal_end_height: 200,
        participants: vec![],
        value_template: "bit".to_string(),
        template_params: json!({}),
    };
    
    let vote_id = service.create_vote(config).await.unwrap();
    
    // First commit should succeed
    let result1 = service.commit(&vote_id, "alice", json!(true), "salt".to_string()).await;
    assert!(result1.is_ok());
    
    // Second commit with same voter should fail (conflict)
    let result2 = service.commit(&vote_id, "alice", json!(false), "salt2".to_string()).await;
    assert!(result2.is_err());
    
    // First reveal should succeed
    let result3 = service.reveal(&vote_id, "alice", json!(true), "salt".to_string()).await;
    assert!(result3.is_ok());
    
    // Second reveal with same voter should fail (conflict)
    let result4 = service.reveal(&vote_id, "alice", json!(true), "salt".to_string()).await;
    assert!(result4.is_err());
}

#[tokio::test]
async fn test_commitment_mismatch() {
    let service = create_test_service().await;
    
    let config = VoteConfig {
        title: "Commitment Test".to_string(),
        description: None,
        options: vec!["Option 1".to_string()],
        commit_start_height: 0,
        commit_end_height: 100,
        reveal_start_height: 101,
        reveal_end_height: 200,
        participants: vec![],
        value_template: "bit".to_string(),
        template_params: json!({}),
    };
    
    let vote_id = service.create_vote(config).await.unwrap();
    
    // Commit with value=true, salt=salt1
    service.commit(&vote_id, "alice", json!(true), "salt1".to_string()).await.unwrap();
    
    // Try to reveal with value=false, salt=salt1 (should fail)
    let result = service.reveal(&vote_id, "alice", json!(false), "salt1".to_string()).await;
    assert!(result.is_err());
    
    // Try to reveal with value=true, salt=salt2 (should fail)
    let result = service.reveal(&vote_id, "alice", json!(true), "salt2".to_string()).await;
    assert!(result.is_err());
    
    // Correct reveal should succeed
    let result = service.reveal(&vote_id, "alice", json!(true), "salt1".to_string()).await;
    assert!(result.is_ok());
}
