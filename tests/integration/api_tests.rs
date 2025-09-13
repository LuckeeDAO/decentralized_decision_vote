use std::sync::Arc;
use tokio::time::{sleep, Duration};
use shared_types::*;
use vote_engine::*;

// Integration tests for API services
mod common;

#[tokio::test]
async fn test_vote_lifecycle_integration() {
    // This test simulates a complete vote lifecycle
    let test_env = common::TestEnvironment::new().await;
    
    // 1. Create a vote
    let config = VoteConfig {
        title: "Integration Test Vote".to_string(),
        description: "A vote for integration testing".to_string(),
        template_id: "yes_no".to_string(),
        template_params: serde_json::json!({
            "options": ["yes", "no"]
        }),
        commitment_duration_hours: 1,
        reveal_duration_hours: 1,
    };
    
    let vote_id = test_env.vote_engine.create_vote(config).await.unwrap();
    assert!(!vote_id.is_empty());
    
    // 2. Get the vote
    let vote = test_env.vote_engine.get_vote(&vote_id).await.unwrap();
    assert_eq!(vote.title, "Integration Test Vote");
    assert_eq!(vote.status, VoteStatus::Created);
    
    // 3. Commit votes
    let voters = vec!["voter1", "voter2", "voter3"];
    let commitments = vec![];
    
    for voter in &voters {
        let commitment_hash = format!("commitment_{}", voter);
        let salt = format!("salt_{}", voter);
        
        let request = CommitRequest {
            voter: voter.to_string(),
            commitment_hash,
            salt,
        };
        
        let response = test_env.vote_engine.commit_vote(&vote_id, request).await.unwrap();
        assert!(response.success);
        commitments.push(response.commitment_id);
    }
    
    // 4. Wait for commitment phase to end (in real scenario)
    // For testing, we'll simulate this by updating the vote status
    
    // 5. Reveal votes
    let reveals = vec!["yes", "no", "yes"];
    
    for (i, voter) in voters.iter().enumerate() {
        let request = RevealRequest {
            voter: voter.to_string(),
            value: serde_json::json!(reveals[i]),
            salt: format!("salt_{}", voter),
        };
        
        let response = test_env.vote_engine.reveal_vote(&vote_id, request).await.unwrap();
        assert!(response.success);
    }
    
    // 6. Get results
    let results = test_env.vote_engine.get_results(&vote_id).await.unwrap();
    assert_eq!(results.total_votes, 3);
    
    // Verify results structure
    let results_data = results.results.as_object().unwrap();
    assert!(results_data.contains_key("yes"));
    assert!(results_data.contains_key("no"));
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_votes() {
    // Test handling multiple votes concurrently
    let test_env = common::TestEnvironment::new().await;
    
    let mut handles = vec![];
    
    // Create 5 votes concurrently
    for i in 0..5 {
        let engine = test_env.vote_engine.clone();
        let handle = tokio::spawn(async move {
            let config = VoteConfig {
                title: format!("Concurrent Vote {}", i),
                description: format!("Vote number {}", i),
                template_id: "yes_no".to_string(),
                template_params: serde_json::json!({}),
                commitment_duration_hours: 1,
                reveal_duration_hours: 1,
            };
            
            engine.create_vote(config).await
        });
        handles.push(handle);
    }
    
    // Wait for all votes to be created
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        let vote_id = result.unwrap().unwrap();
        assert!(!vote_id.is_empty());
    }
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_vote_listing_and_pagination() {
    let test_env = common::TestEnvironment::new().await;
    
    // Create multiple votes
    let mut vote_ids = vec![];
    for i in 0..10 {
        let config = VoteConfig {
            title: format!("List Test Vote {}", i),
            description: format!("Vote for listing test {}", i),
            template_id: "yes_no".to_string(),
            template_params: serde_json::json!({}),
            commitment_duration_hours: 1,
            reveal_duration_hours: 1,
        };
        
        let vote_id = test_env.vote_engine.create_vote(config).await.unwrap();
        vote_ids.push(vote_id);
    }
    
    // Test pagination
    let query = ListQuery {
        page: 1,
        page_size: 5,
        status: None,
        template_id: None,
        created_after: None,
        created_before: None,
    };
    
    let page = test_env.vote_engine.list_votes(query).await.unwrap();
    assert_eq!(page.items.len(), 5);
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 5);
    assert!(page.total >= 10);
    
    // Test second page
    let query2 = ListQuery {
        page: 2,
        page_size: 5,
        status: None,
        template_id: None,
        created_after: None,
        created_before: None,
    };
    
    let page2 = test_env.vote_engine.list_votes(query2).await.unwrap();
    assert_eq!(page2.items.len(), 5);
    assert_eq!(page2.page, 2);
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_error_handling() {
    let test_env = common::TestEnvironment::new().await;
    
    // Test getting non-existent vote
    let result = test_env.vote_engine.get_vote("non-existent").await;
    assert!(result.is_err());
    
    // Test committing to non-existent vote
    let request = CommitRequest {
        voter: "test_voter".to_string(),
        commitment_hash: "test_hash".to_string(),
        salt: "test_salt".to_string(),
    };
    
    let result = test_env.vote_engine.commit_vote("non-existent", request).await;
    assert!(result.is_err());
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_vote_validation() {
    let test_env = common::TestEnvironment::new().await;
    
    // Test invalid vote config
    let invalid_config = VoteConfig {
        title: "".to_string(), // Empty title should fail
        description: "Test".to_string(),
        template_id: "yes_no".to_string(),
        template_params: serde_json::json!({}),
        commitment_duration_hours: 0, // Invalid duration
        reveal_duration_hours: 0,
    };
    
    let result = test_env.vote_engine.create_vote(invalid_config).await;
    assert!(result.is_err());
    
    test_env.cleanup().await;
}
