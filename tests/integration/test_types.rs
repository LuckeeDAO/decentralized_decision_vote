//! 测试中使用的数据类型定义

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_id: String,
    pub title: String,
    pub description: String,
    pub participants: Vec<String>,
    pub commit_deadline: u64,
    pub reveal_deadline: u64,
    pub selection_algorithm: SelectionAlgorithm,
    pub created_at: u64,
}

/// 选择算法类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionAlgorithm {
    Random,
    WeightedRandom,
    RouletteWheel,
    Tournament,
}

/// 揭示数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealData {
    pub participant: String,
    pub randomness: String,
    pub salt: Vec<u8>,
    pub timestamp: u64,
}

/// 选择结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub session_id: String,
    pub winner: String,
    pub total_participants: usize,
    pub selected_count: usize,
    pub random_seed: String,
    pub selection_timestamp: u64,
    pub verification_proof: String,
}

/// 承诺数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentData {
    pub participant: String,
    pub commitment: String,
    pub timestamp: u64,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Created,
    CommitPhase,
    RevealPhase,
    Completed,
    Failed,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub config: SessionConfig,
    pub status: SessionStatus,
    pub commitments: std::collections::HashMap<String, CommitmentData>,
    pub reveals: std::collections::HashMap<String, RevealData>,
    pub result: Option<SelectionResult>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestError {
    SessionNotFound,
    InvalidParticipant,
    CommitmentMismatch,
    Timeout,
    InvalidRandomness,
    SelectionFailed,
    VerificationFailed,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::SessionNotFound => write!(f, "Session not found"),
            TestError::InvalidParticipant => write!(f, "Invalid participant"),
            TestError::CommitmentMismatch => write!(f, "Commitment mismatch"),
            TestError::Timeout => write!(f, "Operation timeout"),
            TestError::InvalidRandomness => write!(f, "Invalid randomness"),
            TestError::SelectionFailed => write!(f, "Selection failed"),
            TestError::VerificationFailed => write!(f, "Verification failed"),
        }
    }
}

impl std::error::Error for TestError {}

/// 测试结果统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatistics {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub execution_time_ms: u64,
    pub winner_distribution: std::collections::HashMap<String, usize>,
    pub average_selection_time_ms: f64,
}

/// 性能测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestConfig {
    pub participant_counts: Vec<usize>,
    pub iterations_per_count: usize,
    pub max_execution_time_ms: u64,
    pub memory_limit_mb: usize,
}

/// 压力测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub participant_count: usize,
    pub successful_selections: usize,
    pub failed_selections: usize,
    pub average_time_ms: f64,
    pub max_time_ms: u64,
    pub min_time_ms: u64,
    pub memory_usage_mb: f64,
}
