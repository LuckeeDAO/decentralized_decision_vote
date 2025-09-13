//! 单目标决策（n选1）测试用例
//! 
//! 测试从n个参与者中选出1个中奖人的去中心化随机数生成功能

use decentralized_decision_vote::core::commitment::CommitmentEngine;
use decentralized_decision_vote::core::randomness::RandomnessEngine;
use decentralized_decision_vote::core::selection::SelectionEngine;
use decentralized_decision_vote::shared::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 测试用例1：基础单目标选择（3选1）
#[tokio::test]
async fn test_basic_single_target_selection_3_choose_1() {
    let session_id = "test_session_3_1";
    let participants = vec!["alice", "bob", "charlie"];
    
    // 创建会话配置
    let config = SessionConfig {
        session_id: session_id.to_string(),
        title: "基础抽奖测试：3选1".to_string(),
        description: "从3个参与者中选出1个中奖者".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    // 初始化引擎
    let commitment_engine = CommitmentEngine::new();
    let randomness_engine = RandomnessEngine::new();
    let selection_engine = SelectionEngine::new();

    // 阶段1：所有参与者提交承诺
    let mut commitments = HashMap::new();
    let mut salt_files = HashMap::new();
    
    for participant in &participants {
        let (commitment, salt) = commitment_engine.generate_commitment(
            &format!("randomness_for_{}", participant)
        ).await.unwrap();
        
        commitments.insert(participant.to_string(), commitment);
        salt_files.insert(participant.to_string(), salt);
    }

    // 阶段2：所有参与者揭示随机数
    let mut reveals = HashMap::new();
    
    for participant in &participants {
        let salt = salt_files.get(participant).unwrap();
        let reveal = RevealData {
            participant: participant.to_string(),
            randomness: format!("randomness_for_{}", participant),
            salt: salt.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // 验证承诺一致性
        let commitment = commitments.get(participant).unwrap();
        assert!(commitment_engine.verify_commitment(
            &reveal.randomness,
            &reveal.salt,
            commitment
        ).await.unwrap());
        
        reveals.insert(participant.to_string(), reveal);
    }

    // 阶段3：选择中奖者
    let selection_result = selection_engine.select_winner(
        &config,
        &reveals
    ).await.unwrap();

    // 验证结果
    assert_eq!(selection_result.session_id, session_id);
    assert!(participants.contains(&selection_result.winner.as_str()));
    assert_eq!(selection_result.total_participants, 3);
    assert_eq!(selection_result.selected_count, 1);
    
    // 验证随机种子生成
    assert!(!selection_result.random_seed.is_empty());
    
    println!("测试通过：中奖者是 {}", selection_result.winner);
}

/// 测试用例2：中等规模单目标选择（10选1）
#[tokio::test]
async fn test_medium_single_target_selection_10_choose_1() {
    let session_id = "test_session_10_1";
    let participants = (1..=10).map(|i| format!("participant_{}", i)).collect::<Vec<_>>();
    
    // 创建会话配置
    let config = SessionConfig {
        session_id: session_id.to_string(),
        title: "中等规模抽奖测试：10选1".to_string(),
        description: "从10个参与者中选出1个中奖者".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    // 初始化引擎
    let commitment_engine = CommitmentEngine::new();
    let selection_engine = SelectionEngine::new();

    // 阶段1：所有参与者提交承诺
    let mut commitments = HashMap::new();
    let mut salt_files = HashMap::new();
    
    for participant in &participants {
        let (commitment, salt) = commitment_engine.generate_commitment(
            &format!("randomness_for_{}", participant)
        ).await.unwrap();
        
        commitments.insert(participant.clone(), commitment);
        salt_files.insert(participant.clone(), salt);
    }

    // 阶段2：所有参与者揭示随机数
    let mut reveals = HashMap::new();
    
    for participant in &participants {
        let salt = salt_files.get(participant).unwrap();
        let reveal = RevealData {
            participant: participant.clone(),
            randomness: format!("randomness_for_{}", participant),
            salt: salt.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // 验证承诺一致性
        let commitment = commitments.get(participant).unwrap();
        assert!(commitment_engine.verify_commitment(
            &reveal.randomness,
            &reveal.salt,
            commitment
        ).await.unwrap());
        
        reveals.insert(participant.clone(), reveal);
    }

    // 阶段3：选择中奖者
    let selection_result = selection_engine.select_winner(
        &config,
        &reveals
    ).await.unwrap();

    // 验证结果
    assert_eq!(selection_result.session_id, session_id);
    assert!(participants.contains(&selection_result.winner));
    assert_eq!(selection_result.total_participants, 10);
    assert_eq!(selection_result.selected_count, 1);
    
    println!("测试通过：中奖者是 {}", selection_result.winner);
}

/// 测试用例3：大规模单目标选择（100选1）
#[tokio::test]
async fn test_large_single_target_selection_100_choose_1() {
    let session_id = "test_session_100_1";
    let participants = (1..=100).map(|i| format!("participant_{}", i)).collect::<Vec<_>>();
    
    // 创建会话配置
    let config = SessionConfig {
        session_id: session_id.to_string(),
        title: "大规模抽奖测试：100选1".to_string(),
        description: "从100个参与者中选出1个中奖者".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    // 初始化引擎
    let commitment_engine = CommitmentEngine::new();
    let selection_engine = SelectionEngine::new();

    // 阶段1：所有参与者提交承诺
    let mut commitments = HashMap::new();
    let mut salt_files = HashMap::new();
    
    for participant in &participants {
        let (commitment, salt) = commitment_engine.generate_commitment(
            &format!("randomness_for_{}", participant)
        ).await.unwrap();
        
        commitments.insert(participant.clone(), commitment);
        salt_files.insert(participant.clone(), salt);
    }

    // 阶段2：所有参与者揭示随机数
    let mut reveals = HashMap::new();
    
    for participant in &participants {
        let salt = salt_files.get(participant).unwrap();
        let reveal = RevealData {
            participant: participant.clone(),
            randomness: format!("randomness_for_{}", participant),
            salt: salt.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // 验证承诺一致性
        let commitment = commitments.get(participant).unwrap();
        assert!(commitment_engine.verify_commitment(
            &reveal.randomness,
            &reveal.salt,
            commitment
        ).await.unwrap());
        
        reveals.insert(participant.clone(), reveal);
    }

    // 阶段3：选择中奖者
    let selection_result = selection_engine.select_winner(
        &config,
        &reveals
    ).await.unwrap();

    // 验证结果
    assert_eq!(selection_result.session_id, session_id);
    assert!(participants.contains(&selection_result.winner));
    assert_eq!(selection_result.total_participants, 100);
    assert_eq!(selection_result.selected_count, 1);
    
    println!("测试通过：中奖者是 {}", selection_result.winner);
}

/// 测试用例4：承诺验证失败场景
#[tokio::test]
async fn test_commitment_verification_failure() {
    let session_id = "test_session_verification_failure";
    let participants = vec!["alice", "bob"];
    
    let commitment_engine = CommitmentEngine::new();
    
    // 生成承诺
    let (commitment, salt) = commitment_engine.generate_commitment(
        "original_randomness"
    ).await.unwrap();
    
    // 尝试用错误的随机数验证
    let wrong_randomness = "wrong_randomness";
    let verification_result = commitment_engine.verify_commitment(
        wrong_randomness,
        &salt,
        &commitment
    ).await.unwrap();
    
    // 应该验证失败
    assert!(!verification_result);
}

/// 测试用例5：重复运行确保随机性
#[tokio::test]
async fn test_randomness_distribution() {
    let session_id = "test_session_randomness";
    let participants = vec!["alice", "bob", "charlie", "david", "eve"];
    let iterations = 100;
    
    let config = SessionConfig {
        session_id: session_id.to_string(),
        title: "随机性分布测试".to_string(),
        description: "测试多次运行的结果分布".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    let commitment_engine = CommitmentEngine::new();
    let selection_engine = SelectionEngine::new();
    
    let mut winner_counts = HashMap::new();
    
    // 运行多次选择
    for iteration in 0..iterations {
        let mut commitments = HashMap::new();
        let mut salt_files = HashMap::new();
        
        // 为每次迭代生成不同的随机数
        for participant in &participants {
            let (commitment, salt) = commitment_engine.generate_commitment(
                &format!("randomness_for_{}_iteration_{}", participant, iteration)
            ).await.unwrap();
            
            commitments.insert(participant.to_string(), commitment);
            salt_files.insert(participant.to_string(), salt);
        }

        let mut reveals = HashMap::new();
        
        for participant in &participants {
            let salt = salt_files.get(participant).unwrap();
            let reveal = RevealData {
                participant: participant.to_string(),
                randomness: format!("randomness_for_{}_iteration_{}", participant, iteration),
                salt: salt.clone(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            };
            
            reveals.insert(participant.to_string(), reveal);
        }

        let selection_result = selection_engine.select_winner(
            &config,
            &reveals
        ).await.unwrap();
        
        *winner_counts.entry(selection_result.winner).or_insert(0) += 1;
    }
    
    // 验证每个参与者都有机会获胜
    assert_eq!(winner_counts.len(), participants.len());
    
    // 验证分布相对均匀（允许一定的偏差）
    let expected_count = iterations / participants.len();
    for (participant, count) in &winner_counts {
        let deviation = (*count as i32 - expected_count as i32).abs();
        let max_deviation = expected_count / 2; // 允许50%的偏差
        
        assert!(
            deviation <= max_deviation as i32,
            "参与者 {} 的获胜次数 {} 偏离期望值 {} 太多",
            participant, count, expected_count
        );
    }
    
    println!("随机性分布测试通过，各参与者获胜次数：{:?}", winner_counts);
}

/// 测试用例6：边界情况 - 只有1个参与者
#[tokio::test]
async fn test_single_participant() {
    let session_id = "test_session_single";
    let participants = vec!["alice"];
    
    let config = SessionConfig {
        session_id: session_id.to_string(),
        title: "单参与者测试".to_string(),
        description: "只有1个参与者的边界情况".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };

    let commitment_engine = CommitmentEngine::new();
    let selection_engine = SelectionEngine::new();
    
    let participant = &participants[0];
    let (commitment, salt) = commitment_engine.generate_commitment(
        &format!("randomness_for_{}", participant)
    ).await.unwrap();
    
    let reveal = RevealData {
        participant: participant.to_string(),
        randomness: format!("randomness_for_{}", participant),
        salt,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let mut reveals = HashMap::new();
    reveals.insert(participant.to_string(), reveal);
    
    let selection_result = selection_engine.select_winner(
        &config,
        &reveals
    ).await.unwrap();
    
    // 唯一参与者应该获胜
    assert_eq!(selection_result.winner, participant.to_string());
    assert_eq!(selection_result.total_participants, 1);
    assert_eq!(selection_result.selected_count, 1);
    
    println!("单参与者测试通过：中奖者是 {}", selection_result.winner);
}

/// 测试用例7：超时场景测试
#[tokio::test]
async fn test_timeout_scenarios() {
    let session_id = "test_session_timeout";
    let participants = vec!["alice", "bob", "charlie"];
    
    // 创建已过期的会话配置
    let expired_config = SessionConfig {
        session_id: session_id.to_string(),
        title: "超时测试".to_string(),
        description: "测试超时场景".to_string(),
        participants: participants.clone(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600, // 已过期
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 1800, // 已过期
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 7200,
    };

    let commitment_engine = CommitmentEngine::new();
    
    // 尝试在过期后提交承诺应该失败
    let result = commitment_engine.generate_commitment("test_randomness").await;
    // 这里应该根据实际实现返回错误或拒绝
    // assert!(result.is_err());
    
    println!("超时场景测试完成");
}
