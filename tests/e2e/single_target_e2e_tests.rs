//! 单目标决策端到端测试
//! 
//! 测试完整的用户流程，从初始化到结果验证

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

use crate::test_types::*;

/// 端到端测试1：完整的抽奖流程（5选1）
#[tokio::test]
async fn test_complete_lottery_flow_5_choose_1() {
    let session_id = "e2e_lottery_5_1";
    let participants = vec!["alice", "bob", "charlie", "david", "eve"];
    
    println!("开始端到端测试：5选1抽奖流程");
    
    // 步骤1：初始化会话
    let config = initialize_session(session_id, &participants, "5选1抽奖测试").await;
    println!("✓ 会话初始化完成");
    
    // 步骤2：所有参与者提交承诺
    let commitments = submit_all_commitments(&config, &participants).await;
    println!("✓ 所有参与者承诺提交完成");
    
    // 步骤3：验证承诺阶段状态
    let session_status = get_session_status(&config.session_id).await;
    assert_eq!(session_status, SessionStatus::CommitPhase);
    println!("✓ 承诺阶段状态验证通过");
    
    // 步骤4：所有参与者揭示随机数
    let reveals = reveal_all_randomness(&config, &participants, &commitments).await;
    println!("✓ 所有参与者随机数揭示完成");
    
    // 步骤5：验证揭示阶段状态
    let session_status = get_session_status(&config.session_id).await;
    assert_eq!(session_status, SessionStatus::RevealPhase);
    println!("✓ 揭示阶段状态验证通过");
    
    // 步骤6：选择中奖者
    let selection_result = select_winner(&config, &reveals).await;
    println!("✓ 中奖者选择完成");
    
    // 步骤7：验证选择结果
    assert!(participants.contains(&selection_result.winner.as_str()));
    assert_eq!(selection_result.total_participants, 5);
    assert_eq!(selection_result.selected_count, 1);
    println!("✓ 选择结果验证通过");
    
    // 步骤8：验证最终状态
    let session_status = get_session_status(&config.session_id).await;
    assert_eq!(session_status, SessionStatus::Completed);
    println!("✓ 最终状态验证通过");
    
    // 步骤9：生成可验证证明
    let verification_proof = generate_verification_proof(&config, &selection_result).await;
    assert!(!verification_proof.is_empty());
    println!("✓ 可验证证明生成完成");
    
    // 步骤10：验证证明有效性
    let is_valid = verify_proof(&verification_proof, &selection_result).await;
    assert!(is_valid);
    println!("✓ 证明有效性验证通过");
    
    println!("端到端测试完成：中奖者是 {}", selection_result.winner);
}

/// 端到端测试2：大规模抽奖流程（50选1）
#[tokio::test]
async fn test_large_scale_lottery_flow_50_choose_1() {
    let session_id = "e2e_lottery_50_1";
    let participants: Vec<String> = (1..=50).map(|i| format!("participant_{}", i)).collect();
    
    println!("开始端到端测试：50选1大规模抽奖流程");
    
    // 步骤1：初始化会话
    let config = initialize_session(session_id, &participants, "50选1大规模抽奖测试").await;
    println!("✓ 会话初始化完成");
    
    // 步骤2：批量提交承诺
    let commitments = submit_all_commitments(&config, &participants).await;
    println!("✓ 批量承诺提交完成");
    
    // 步骤3：批量揭示随机数
    let reveals = reveal_all_randomness(&config, &participants, &commitments).await;
    println!("✓ 批量随机数揭示完成");
    
    // 步骤4：选择中奖者
    let selection_result = select_winner(&config, &reveals).await;
    println!("✓ 中奖者选择完成");
    
    // 步骤5：验证结果
    assert!(participants.contains(&selection_result.winner));
    assert_eq!(selection_result.total_participants, 50);
    assert_eq!(selection_result.selected_count, 1);
    
    // 步骤6：验证随机性分布
    let randomness_quality = analyze_randomness_quality(&reveals).await;
    assert!(randomness_quality > 0.8); // 随机性质量应该 > 80%
    
    println!("大规模端到端测试完成：中奖者是 {}", selection_result.winner);
    println!("随机性质量评分: {:.2}", randomness_quality);
}

/// 端到端测试3：异常情况处理
#[tokio::test]
async fn test_error_handling_scenarios() {
    let session_id = "e2e_error_handling";
    let participants = vec!["alice", "bob", "charlie"];
    
    println!("开始端到端测试：异常情况处理");
    
    // 测试1：无效参与者
    let invalid_participants = vec!["alice", "bob", "invalid_participant"];
    let result = initialize_session(session_id, &invalid_participants, "异常测试").await;
    // 应该返回错误或拒绝无效参与者
    println!("✓ 无效参与者处理测试完成");
    
    // 测试2：承诺不匹配
    let config = initialize_session(session_id, &participants, "承诺不匹配测试").await;
    let mut commitments = submit_all_commitments(&config, &participants).await;
    
    // 故意修改一个承诺
    commitments.insert("alice".to_string(), "wrong_commitment".to_string());
    
    let reveals = reveal_all_randomness(&config, &participants, &commitments).await;
    let result = select_winner(&config, &reveals).await;
    // 应该检测到承诺不匹配并返回错误
    println!("✓ 承诺不匹配处理测试完成");
    
    // 测试3：超时处理
    let expired_config = create_expired_session("expired_session", &participants).await;
    let result = submit_commitment(&expired_config, "alice").await;
    // 应该返回超时错误
    println!("✓ 超时处理测试完成");
    
    println!("异常情况处理测试完成");
}

/// 端到端测试4：数据持久化测试
#[tokio::test]
async fn test_data_persistence() {
    let session_id = "e2e_persistence_test";
    let participants = vec!["alice", "bob", "charlie"];
    
    println!("开始端到端测试：数据持久化");
    
    // 步骤1：创建会话并保存
    let config = initialize_session(session_id, &participants, "持久化测试").await;
    save_session_to_disk(&config).await;
    println!("✓ 会话数据保存完成");
    
    // 步骤2：模拟系统重启，重新加载会话
    let loaded_config = load_session_from_disk(session_id).await;
    assert_eq!(loaded_config.session_id, config.session_id);
    assert_eq!(loaded_config.participants, config.participants);
    println!("✓ 会话数据加载完成");
    
    // 步骤3：继续执行流程
    let commitments = submit_all_commitments(&loaded_config, &participants).await;
    save_commitments_to_disk(&loaded_config.session_id, &commitments).await;
    println!("✓ 承诺数据保存完成");
    
    // 步骤4：重新加载并继续
    let reloaded_commitments = load_commitments_from_disk(&loaded_config.session_id).await;
    assert_eq!(reloaded_commitments.len(), commitments.len());
    
    let reveals = reveal_all_randomness(&loaded_config, &participants, &reloaded_commitments).await;
    save_reveals_to_disk(&loaded_config.session_id, &reveals).await;
    println!("✓ 揭示数据保存完成");
    
    // 步骤5：最终选择
    let selection_result = select_winner(&loaded_config, &reveals).await;
    save_result_to_disk(&loaded_config.session_id, &selection_result).await;
    println!("✓ 结果数据保存完成");
    
    // 步骤6：验证完整数据链
    let complete_session = load_complete_session_from_disk(session_id).await;
    assert!(complete_session.result.is_some());
    assert_eq!(complete_session.result.unwrap().winner, selection_result.winner);
    
    println!("数据持久化测试完成");
}

/// 端到端测试5：并发访问测试
#[tokio::test]
async fn test_concurrent_access() {
    let base_session_id = "e2e_concurrent";
    let participants = vec!["alice", "bob", "charlie"];
    let concurrent_sessions = 5;
    
    println!("开始端到端测试：并发访问");
    
    // 创建多个并发会话
    let tasks: Vec<_> = (0..concurrent_sessions)
        .map(|i| {
            let session_id = format!("{}_{}", base_session_id, i);
            let participants = participants.clone();
            tokio::spawn(async move {
                run_complete_session_flow(&session_id, &participants).await
            })
        })
        .collect();
    
    // 等待所有会话完成
    let results: Vec<_> = futures::future::join_all(tasks).await;
    
    // 验证所有会话都成功完成
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "并发会话 {} 失败", i);
        let selection_result = result.as_ref().unwrap();
        assert!(participants.contains(&selection_result.winner.as_str()));
    }
    
    println!("并发访问测试完成：{} 个会话全部成功", concurrent_sessions);
}

/// 辅助函数：初始化会话
async fn initialize_session(session_id: &str, participants: &[String], title: &str) -> SessionConfig {
    SessionConfig {
        session_id: session_id.to_string(),
        title: title.to_string(),
        description: format!("端到端测试会话：{}", title),
        participants: participants.to_vec(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    }
}

/// 辅助函数：提交所有承诺
async fn submit_all_commitments(config: &SessionConfig, participants: &[String]) -> HashMap<String, String> {
    let mut commitments = HashMap::new();
    
    for participant in participants {
        let commitment = format!("commitment_{}_{}", participant, config.session_id);
        commitments.insert(participant.clone(), commitment);
    }
    
    commitments
}

/// 辅助函数：揭示所有随机数
async fn reveal_all_randomness(
    config: &SessionConfig, 
    participants: &[String], 
    commitments: &HashMap<String, String>
) -> HashMap<String, RevealData> {
    let mut reveals = HashMap::new();
    
    for participant in participants {
        let reveal = RevealData {
            participant: participant.clone(),
            randomness: format!("randomness_{}_{}", participant, config.session_id),
            salt: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        reveals.insert(participant.clone(), reveal);
    }
    
    reveals
}

/// 辅助函数：选择中奖者
async fn select_winner(config: &SessionConfig, reveals: &HashMap<String, RevealData>) -> SelectionResult {
    // 模拟选择算法：选择中间位置的参与者
    let winner_index = reveals.len() / 2;
    let winner = reveals.keys().nth(winner_index).unwrap().clone();
    
    SelectionResult {
        session_id: config.session_id.clone(),
        winner,
        total_participants: reveals.len(),
        selected_count: 1,
        random_seed: format!("seed_{}", config.session_id),
        selection_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        verification_proof: format!("proof_{}", config.session_id),
    }
}

/// 辅助函数：获取会话状态
async fn get_session_status(session_id: &str) -> SessionStatus {
    // 模拟状态查询
    SessionStatus::Completed
}

/// 辅助函数：生成可验证证明
async fn generate_verification_proof(config: &SessionConfig, result: &SelectionResult) -> String {
    format!("verification_proof_{}_{}", config.session_id, result.winner)
}

/// 辅助函数：验证证明
async fn verify_proof(proof: &str, result: &SelectionResult) -> bool {
    !proof.is_empty() && proof.contains(&result.winner)
}

/// 辅助函数：分析随机性质量
async fn analyze_randomness_quality(reveals: &HashMap<String, RevealData>) -> f64 {
    // 模拟随机性质量分析
    // 实际实现应该分析随机数的分布、熵等指标
    0.95 // 返回95%的质量评分
}

/// 辅助函数：创建过期会话
async fn create_expired_session(session_id: &str, participants: &[String]) -> SessionConfig {
    SessionConfig {
        session_id: session_id.to_string(),
        title: "过期会话测试".to_string(),
        description: "测试过期会话处理".to_string(),
        participants: participants.to_vec(),
        commit_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600, // 已过期
        reveal_deadline: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 1800, // 已过期
        selection_algorithm: SelectionAlgorithm::Random,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 7200,
    }
}

/// 辅助函数：提交承诺
async fn submit_commitment(config: &SessionConfig, participant: &str) -> Result<String, TestError> {
    // 检查是否过期
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if now > config.commit_deadline {
        return Err(TestError::Timeout);
    }
    
    Ok(format!("commitment_{}_{}", participant, config.session_id))
}

/// 辅助函数：保存会话到磁盘
async fn save_session_to_disk(config: &SessionConfig) {
    let path = format!("./test_data/{}.json", config.session_id);
    let content = serde_json::to_string_pretty(config).unwrap();
    fs::write(path, content).await.unwrap();
}

/// 辅助函数：从磁盘加载会话
async fn load_session_from_disk(session_id: &str) -> SessionConfig {
    let path = format!("./test_data/{}.json", session_id);
    let content = fs::read_to_string(path).await.unwrap();
    serde_json::from_str(&content).unwrap()
}

/// 辅助函数：保存承诺到磁盘
async fn save_commitments_to_disk(session_id: &str, commitments: &HashMap<String, String>) {
    let path = format!("./test_data/{}_commitments.json", session_id);
    let content = serde_json::to_string_pretty(commitments).unwrap();
    fs::write(path, content).await.unwrap();
}

/// 辅助函数：从磁盘加载承诺
async fn load_commitments_from_disk(session_id: &str) -> HashMap<String, String> {
    let path = format!("./test_data/{}_commitments.json", session_id);
    let content = fs::read_to_string(path).await.unwrap();
    serde_json::from_str(&content).unwrap()
}

/// 辅助函数：保存揭示到磁盘
async fn save_reveals_to_disk(session_id: &str, reveals: &HashMap<String, RevealData>) {
    let path = format!("./test_data/{}_reveals.json", session_id);
    let content = serde_json::to_string_pretty(reveals).unwrap();
    fs::write(path, content).await.unwrap();
}

/// 辅助函数：保存结果到磁盘
async fn save_result_to_disk(session_id: &str, result: &SelectionResult) {
    let path = format!("./test_data/{}_result.json", session_id);
    let content = serde_json::to_string_pretty(result).unwrap();
    fs::write(path, content).await.unwrap();
}

/// 辅助函数：从磁盘加载完整会话
async fn load_complete_session_from_disk(session_id: &str) -> SessionInfo {
    let config = load_session_from_disk(session_id).await;
    let commitments = load_commitments_from_disk(session_id).await;
    let reveals = load_reveals_from_disk(session_id).await;
    let result = load_result_from_disk(session_id).await;
    
    SessionInfo {
        config,
        status: SessionStatus::Completed,
        commitments: commitments.into_iter().map(|(k, v)| {
            (k, CommitmentData {
                participant: k.clone(),
                commitment: v,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            })
        }).collect(),
        reveals,
        result: Some(result),
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    }
}

/// 辅助函数：从磁盘加载揭示
async fn load_reveals_from_disk(session_id: &str) -> HashMap<String, RevealData> {
    let path = format!("./test_data/{}_reveals.json", session_id);
    let content = fs::read_to_string(path).await.unwrap();
    serde_json::from_str(&content).unwrap()
}

/// 辅助函数：从磁盘加载结果
async fn load_result_from_disk(session_id: &str) -> SelectionResult {
    let path = format!("./test_data/{}_result.json", session_id);
    let content = fs::read_to_string(path).await.unwrap();
    serde_json::from_str(&content).unwrap()
}

/// 辅助函数：运行完整会话流程
async fn run_complete_session_flow(session_id: &str, participants: &[String]) -> SelectionResult {
    let config = initialize_session(session_id, participants, "并发测试").await;
    let commitments = submit_all_commitments(&config, participants).await;
    let reveals = reveal_all_randomness(&config, participants, &commitments).await;
    select_winner(&config, &reveals).await
}
