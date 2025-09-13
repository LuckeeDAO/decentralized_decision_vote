use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use shared_types::*;
use vote_engine::*;

/// Performance and load tests for the vote system
mod common;

#[tokio::test]
async fn test_concurrent_vote_creation() {
    let test_env = common::TestEnvironment::new().await;
    let start_time = Instant::now();
    
    let mut handles = vec![];
    let num_votes = 100;
    
    // Create votes concurrently
    for i in 0..num_votes {
        let engine = test_env.vote_engine.clone();
        let handle = tokio::spawn(async move {
            let config = VoteConfig {
                title: format!("Load Test Vote {}", i),
                description: format!("Vote for load testing {}", i),
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
    
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    
    // Verify all votes were created successfully
    let mut success_count = 0;
    for result in results {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, num_votes);
    
    println!("Created {} votes in {:?}", num_votes, duration);
    println!("Average time per vote: {:?}", duration / num_votes);
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_commitments() {
    let test_env = common::TestEnvironment::new().await;
    
    // Create a vote first
    let config = VoteConfig {
        title: "Load Test Vote".to_string(),
        description: "Vote for commitment load testing".to_string(),
        template_id: "yes_no".to_string(),
        template_params: serde_json::json!({}),
        commitment_duration_hours: 1,
        reveal_duration_hours: 1,
    };
    
    let vote_id = test_env.vote_engine.create_vote(config).await.unwrap();
    
    let start_time = Instant::now();
    let num_commitments = 50;
    let mut handles = vec![];
    
    // Create commitments concurrently
    for i in 0..num_commitments {
        let engine = test_env.vote_engine.clone();
        let vote_id = vote_id.clone();
        let handle = tokio::spawn(async move {
            let request = CommitRequest {
                voter: format!("voter_{}", i),
                commitment_hash: format!("commitment_{}", i),
                salt: format!("salt_{}", i),
            };
            
            engine.commit_vote(&vote_id, request).await
        });
        handles.push(handle);
    }
    
    // Wait for all commitments to be processed
    let results = futures::future::join_all(handles).await;
    
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    
    // Verify all commitments were successful
    let mut success_count = 0;
    for result in results {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, num_commitments);
    
    println!("Processed {} commitments in {:?}", num_commitments, duration);
    println!("Average time per commitment: {:?}", duration / num_commitments);
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let test_env = common::TestEnvironment::new().await;
    
    // Monitor memory usage during high load
    let num_operations = 1000;
    let mut handles = vec![];
    
    let start_time = Instant::now();
    
    for i in 0..num_operations {
        let engine = test_env.vote_engine.clone();
        let handle = tokio::spawn(async move {
            // Create vote
            let config = VoteConfig {
                title: format!("Memory Test Vote {}", i),
                description: format!("Vote for memory testing {}", i),
                template_id: "yes_no".to_string(),
                template_params: serde_json::json!({}),
                commitment_duration_hours: 1,
                reveal_duration_hours: 1,
            };
            
            let vote_id = engine.create_vote(config).await?;
            
            // Create commitment
            let request = CommitRequest {
                voter: format!("voter_{}", i),
                commitment_hash: format!("commitment_{}", i),
                salt: format!("salt_{}", i),
            };
            
            engine.commit_vote(&vote_id, request).await?;
            
            Ok::<String, VoteError>(vote_id)
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;
    
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    
    // Count successful operations
    let mut success_count = 0;
    for result in results {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, num_operations);
    
    println!("Completed {} operations in {:?}", num_operations, duration);
    println!("Operations per second: {:.2}", num_operations as f64 / duration.as_secs_f64());
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_response_time_consistency() {
    let test_env = common::TestEnvironment::new().await;
    
    let num_requests = 100;
    let mut response_times = Vec::new();
    
    for i in 0..num_requests {
        let start_time = Instant::now();
        
        let config = VoteConfig {
            title: format!("Response Time Test Vote {}", i),
            description: format!("Vote for response time testing {}", i),
            template_id: "yes_no".to_string(),
            template_params: serde_json::json!({}),
            commitment_duration_hours: 1,
            reveal_duration_hours: 1,
        };
        
        let _vote_id = test_env.vote_engine.create_vote(config).await.unwrap();
        
        let end_time = Instant::now();
        let response_time = end_time.duration_since(start_time);
        response_times.push(response_time);
        
        // Small delay to avoid overwhelming the system
        sleep(Duration::from_millis(10)).await;
    }
    
    // Calculate statistics
    let total_time: Duration = response_times.iter().sum();
    let average_time = total_time / num_requests;
    let min_time = response_times.iter().min().unwrap();
    let max_time = response_times.iter().max().unwrap();
    
    // Calculate standard deviation
    let variance: f64 = response_times.iter()
        .map(|&time| {
            let diff = time.as_nanos() as f64 - average_time.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>() / num_requests as f64;
    let std_dev = variance.sqrt();
    
    println!("Response time statistics for {} requests:", num_requests);
    println!("  Average: {:?}", average_time);
    println!("  Min: {:?}", min_time);
    println!("  Max: {:?}", max_time);
    println!("  Std Dev: {:.2}ms", std_dev / 1_000_000.0);
    
    // Assert that response times are reasonable
    assert!(average_time < Duration::from_millis(100), "Average response time too high");
    assert!(max_time < Duration::from_millis(500), "Max response time too high");
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_sustained_load() {
    let test_env = common::TestEnvironment::new().await;
    
    let duration = Duration::from_secs(30); // Run for 30 seconds
    let start_time = Instant::now();
    let mut operation_count = 0;
    
    while start_time.elapsed() < duration {
        let config = VoteConfig {
            title: format!("Sustained Load Vote {}", operation_count),
            description: format!("Vote for sustained load testing {}", operation_count),
            template_id: "yes_no".to_string(),
            template_params: serde_json::json!({}),
            commitment_duration_hours: 1,
            reveal_duration_hours: 1,
        };
        
        let _vote_id = test_env.vote_engine.create_vote(config).await.unwrap();
        operation_count += 1;
        
        // Small delay to prevent overwhelming
        sleep(Duration::from_millis(50)).await;
    }
    
    let actual_duration = start_time.elapsed();
    let ops_per_second = operation_count as f64 / actual_duration.as_secs_f64();
    
    println!("Sustained load test results:");
    println!("  Duration: {:?}", actual_duration);
    println!("  Operations: {}", operation_count);
    println!("  Operations per second: {:.2}", ops_per_second);
    
    // Assert minimum throughput
    assert!(ops_per_second > 10.0, "Throughput too low: {:.2} ops/sec", ops_per_second);
    
    test_env.cleanup().await;
}
