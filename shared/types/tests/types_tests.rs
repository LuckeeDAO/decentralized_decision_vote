use shared_types::*;
use serde_json::json;
use chrono::{Utc, Duration};

#[test]
fn test_vote_serialization() {
    let vote = Vote {
        id: "test_vote_1".to_string(),
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "yes_no".to_string(),
        template_params: json!({"options": ["yes", "no"]}),
        creator: "test_user".to_string(),
        created_at: Utc::now(),
        commitment_start: Utc::now(),
        commitment_end: Utc::now() + Duration::hours(24),
        reveal_start: Utc::now() + Duration::hours(24),
        reveal_end: Utc::now() + Duration::hours(48),
        status: VoteStatus::Created,
        results: None,
    };

    // Test serialization
    let serialized = serde_json::to_string(&vote).unwrap();
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: Vote = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, vote.id);
    assert_eq!(deserialized.title, vote.title);
    assert_eq!(deserialized.template_id, vote.template_id);
}

#[test]
fn test_vote_status_serialization() {
    let statuses = vec![
        VoteStatus::Created,
        VoteStatus::CommitmentPhase,
        VoteStatus::RevealPhase,
        VoteStatus::Completed,
        VoteStatus::Cancelled,
    ];

    for status in statuses {
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: VoteStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(format!("{:?}", deserialized), format!("{:?}", status));
    }
}

#[test]
fn test_vote_config_validation() {
    let config = VoteConfig {
        title: "Test Vote".to_string(),
        description: "A test vote".to_string(),
        template_id: "yes_no".to_string(),
        template_params: json!({}),
        commitment_duration_hours: 24,
        reveal_duration_hours: 24,
    };

    // Test serialization
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: VoteConfig = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.title, config.title);
    assert_eq!(deserialized.commitment_duration_hours, config.commitment_duration_hours);
}

#[test]
fn test_commitment_serialization() {
    let commitment = Commitment {
        id: "commit_1".to_string(),
        vote_id: "vote_1".to_string(),
        voter: "voter_1".to_string(),
        commitment_hash: "abc123".to_string(),
        salt: "salt123".to_string(),
        created_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&commitment).unwrap();
    let deserialized: Commitment = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.id, commitment.id);
    assert_eq!(deserialized.vote_id, commitment.vote_id);
    assert_eq!(deserialized.commitment_hash, commitment.commitment_hash);
}

#[test]
fn test_reveal_serialization() {
    let reveal = Reveal {
        id: "reveal_1".to_string(),
        vote_id: "vote_1".to_string(),
        voter: "voter_1".to_string(),
        value: json!("yes"),
        salt: "salt123".to_string(),
        created_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&reveal).unwrap();
    let deserialized: Reveal = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.id, reveal.id);
    assert_eq!(deserialized.vote_id, reveal.vote_id);
    assert_eq!(deserialized.value, reveal.value);
}

#[test]
fn test_vote_results_serialization() {
    let results = VoteResults {
        vote_id: "vote_1".to_string(),
        total_votes: 10,
        results: json!({
            "yes": 6,
            "no": 4
        }),
        calculated_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&results).unwrap();
    let deserialized: VoteResults = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.vote_id, results.vote_id);
    assert_eq!(deserialized.total_votes, results.total_votes);
    assert_eq!(deserialized.results, results.results);
}

#[test]
fn test_commit_request_serialization() {
    let request = CommitRequest {
        voter: "voter_1".to_string(),
        commitment_hash: "hash123".to_string(),
        salt: "salt123".to_string(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: CommitRequest = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.voter, request.voter);
    assert_eq!(deserialized.commitment_hash, request.commitment_hash);
}

#[test]
fn test_commit_response_serialization() {
    let response = CommitResponse {
        commitment_id: "commit_1".to_string(),
        success: true,
        message: "Success".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: CommitResponse = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.commitment_id, response.commitment_id);
    assert_eq!(deserialized.success, response.success);
}

#[test]
fn test_reveal_request_serialization() {
    let request = RevealRequest {
        voter: "voter_1".to_string(),
        value: json!("yes"),
        salt: "salt123".to_string(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: RevealRequest = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.voter, request.voter);
    assert_eq!(deserialized.value, request.value);
}

#[test]
fn test_reveal_response_serialization() {
    let response = RevealResponse {
        reveal_id: "reveal_1".to_string(),
        success: true,
        message: "Success".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: RevealResponse = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.reveal_id, response.reveal_id);
    assert_eq!(deserialized.success, response.success);
}

#[test]
fn test_list_query_serialization() {
    let query = ListQuery {
        page: 1,
        page_size: 10,
        status: Some(VoteStatus::Created),
        creator: Some("test_user".to_string()),
    };

    let serialized = serde_json::to_string(&query).unwrap();
    let deserialized: ListQuery = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.page, query.page);
    assert_eq!(deserialized.page_size, query.page_size);
    assert!(deserialized.status.is_some());
    assert!(deserialized.creator.is_some());
}

#[test]
fn test_page_serialization() {
    let items = vec![
        Vote {
            id: "vote_1".to_string(),
            title: "Vote 1".to_string(),
            description: "Description 1".to_string(),
            template_id: "yes_no".to_string(),
            template_params: json!({}),
            creator: "user1".to_string(),
            created_at: Utc::now(),
            commitment_start: Utc::now(),
            commitment_end: Utc::now() + Duration::hours(24),
            reveal_start: Utc::now() + Duration::hours(24),
            reveal_end: Utc::now() + Duration::hours(48),
            status: VoteStatus::Created,
            results: None,
        },
        Vote {
            id: "vote_2".to_string(),
            title: "Vote 2".to_string(),
            description: "Description 2".to_string(),
            template_id: "yes_no".to_string(),
            template_params: json!({}),
            creator: "user2".to_string(),
            created_at: Utc::now(),
            commitment_start: Utc::now(),
            commitment_end: Utc::now() + Duration::hours(24),
            reveal_start: Utc::now() + Duration::hours(24),
            reveal_end: Utc::now() + Duration::hours(48),
            status: VoteStatus::Created,
            results: None,
        },
    ];

    let page = Page {
        items,
        total: 2,
        page: 1,
        page_size: 10,
        total_pages: 1,
    };

    let serialized = serde_json::to_string(&page).unwrap();
    let deserialized: Page<Vote> = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.total, page.total);
    assert_eq!(deserialized.items.len(), page.items.len());
    assert_eq!(deserialized.page, page.page);
}

#[test]
fn test_json_value_handling() {
    // Test various JSON value types in template_params
    let vote = Vote {
        id: "test".to_string(),
        title: "Test".to_string(),
        description: "Test".to_string(),
        template_id: "test".to_string(),
        template_params: json!({
            "string": "value",
            "number": 42,
            "boolean": true,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        }),
        creator: "test".to_string(),
        created_at: Utc::now(),
        commitment_start: Utc::now(),
        commitment_end: Utc::now() + Duration::hours(24),
        reveal_start: Utc::now() + Duration::hours(24),
        reveal_end: Utc::now() + Duration::hours(48),
        status: VoteStatus::Created,
        results: None,
    };

    let serialized = serde_json::to_string(&vote).unwrap();
    let deserialized: Vote = serde_json::from_str(&serialized).unwrap();
    
    // Verify complex JSON structure is preserved
    assert_eq!(deserialized.template_params["string"], "value");
    assert_eq!(deserialized.template_params["number"], 42);
    assert_eq!(deserialized.template_params["boolean"], true);
    assert_eq!(deserialized.template_params["array"], json!([1, 2, 3]));
    assert_eq!(deserialized.template_params["object"]["nested"], "value");
}
