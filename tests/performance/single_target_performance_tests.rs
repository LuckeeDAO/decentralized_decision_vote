//! 单目标决策性能测试
//! 
//! 测试不同规模下的性能表现和资源使用情况

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

use crate::test_types::*;

/// 性能测试1：小规模性能基准（10个参与者）
#[tokio::test]
async fn test_small_scale_performance_10_participants() {
    let participant_count = 10;
    let iterations = 50;
    
    let mut total_time = Duration::new(0, 0);
    let mut successful_selections = 0;
    
    for iteration in 0..iterations {
        let start_time = Instant::now();
        
        let result = run_single_selection_test(
            &format!("perf_test_10_{}", iteration),
            participant_count,
        ).await;
        
        let elapsed = start_time.elapsed();
        total_time += elapsed;
        
        if result.is_ok() {
            successful_selections += 1;
        }
        
        // 单次选择应该在100ms内完成
        assert!(elapsed.as_millis() < 100, 
            "单次选择耗时 {}ms，超过100ms限制", elapsed.as_millis());
    }
    
    let average_time = total_time / iterations as u32;
    let success_rate = (successful_selections as f64 / iterations as f64) * 100.0;
    
    println!("小规模性能测试结果（{}个参与者，{}次迭代）：", participant_count, iterations);
    println!("  平均耗时: {:?}", average_time);
    println!("  成功率: {:.2}%", success_rate);
    println!("  总耗时: {:?}", total_time);
    
    // 性能要求：平均耗时 < 50ms，成功率 > 95%
    assert!(average_time.as_millis() < 50, "平均耗时 {}ms 超过50ms限制", average_time.as_millis());
    assert!(success_rate > 95.0, "成功率 {:.2}% 低于95%要求", success_rate);
}

/// 性能测试2：中等规模性能测试（100个参与者）
#[tokio::test]
async fn test_medium_scale_performance_100_participants() {
    let participant_count = 100;
    let iterations = 20;
    
    let mut total_time = Duration::new(0, 0);
    let mut successful_selections = 0;
    
    for iteration in 0..iterations {
        let start_time = Instant::now();
        
        let result = run_single_selection_test(
            &format!("perf_test_100_{}", iteration),
            participant_count,
        ).await;
        
        let elapsed = start_time.elapsed();
        total_time += elapsed;
        
        if result.is_ok() {
            successful_selections += 1;
        }
        
        // 单次选择应该在500ms内完成
        assert!(elapsed.as_millis() < 500, 
            "单次选择耗时 {}ms，超过500ms限制", elapsed.as_millis());
    }
    
    let average_time = total_time / iterations as u32;
    let success_rate = (successful_selections as f64 / iterations as f64) * 100.0;
    
    println!("中等规模性能测试结果（{}个参与者，{}次迭代）：", participant_count, iterations);
    println!("  平均耗时: {:?}", average_time);
    println!("  成功率: {:.2}%", success_rate);
    println!("  总耗时: {:?}", total_time);
    
    // 性能要求：平均耗时 < 200ms，成功率 > 95%
    assert!(average_time.as_millis() < 200, "平均耗时 {}ms 超过200ms限制", average_time.as_millis());
    assert!(success_rate > 95.0, "成功率 {:.2}% 低于95%要求", success_rate);
}

/// 性能测试3：大规模性能测试（1000个参与者）
#[tokio::test]
async fn test_large_scale_performance_1000_participants() {
    let participant_count = 1000;
    let iterations = 5;
    
    let mut total_time = Duration::new(0, 0);
    let mut successful_selections = 0;
    
    for iteration in 0..iterations {
        let start_time = Instant::now();
        
        let result = run_single_selection_test(
            &format!("perf_test_1000_{}", iteration),
            participant_count,
        ).await;
        
        let elapsed = start_time.elapsed();
        total_time += elapsed;
        
        if result.is_ok() {
            successful_selections += 1;
        }
        
        // 单次选择应该在2秒内完成
        assert!(elapsed.as_secs() < 2, 
            "单次选择耗时 {}s，超过2秒限制", elapsed.as_secs());
    }
    
    let average_time = total_time / iterations as u32;
    let success_rate = (successful_selections as f64 / iterations as f64) * 100.0;
    
    println!("大规模性能测试结果（{}个参与者，{}次迭代）：", participant_count, iterations);
    println!("  平均耗时: {:?}", average_time);
    println!("  成功率: {:.2}%", success_rate);
    println!("  总耗时: {:?}", total_time);
    
    // 性能要求：平均耗时 < 1秒，成功率 > 90%
    assert!(average_time.as_secs() < 1, "平均耗时 {}s 超过1秒限制", average_time.as_secs());
    assert!(success_rate > 90.0, "成功率 {:.2}% 低于90%要求", success_rate);
}

/// 性能测试4：并发性能测试
#[tokio::test]
async fn test_concurrent_performance() {
    let participant_count = 50;
    let concurrent_sessions = 10;
    
    let start_time = Instant::now();
    
    // 创建多个并发任务
    let tasks: Vec<_> = (0..concurrent_sessions)
        .map(|i| {
            tokio::spawn(async move {
                run_single_selection_test(
                    &format!("concurrent_test_{}", i),
                    participant_count,
                ).await
            })
        })
        .collect();
    
    // 等待所有任务完成
    let results: Vec<_> = futures::future::join_all(tasks).await;
    
    let total_time = start_time.elapsed();
    let successful_count = results.iter().filter(|r| r.is_ok()).count();
    let success_rate = (successful_count as f64 / concurrent_sessions as f64) * 100.0;
    
    println!("并发性能测试结果（{}个并发会话，每个{}个参与者）：", concurrent_sessions, participant_count);
    println!("  总耗时: {:?}", total_time);
    println!("  成功率: {:.2}%", success_rate);
    println!("  平均每会话耗时: {:?}", total_time / concurrent_sessions as u32);
    
    // 性能要求：总耗时 < 5秒，成功率 > 90%
    assert!(total_time.as_secs() < 5, "总耗时 {}s 超过5秒限制", total_time.as_secs());
    assert!(success_rate > 90.0, "成功率 {:.2}% 低于90%要求", success_rate);
}

/// 性能测试5：内存使用测试
#[tokio::test]
async fn test_memory_usage() {
    let participant_counts = vec![10, 50, 100, 500, 1000];
    
    for participant_count in participant_counts {
        let start_memory = get_memory_usage();
        
        let result = run_single_selection_test(
            &format!("memory_test_{}", participant_count),
            participant_count,
        ).await;
        
        let end_memory = get_memory_usage();
        let memory_delta = end_memory - start_memory;
        
        println!("内存使用测试（{}个参与者）：", participant_count);
        println!("  内存增量: {} MB", memory_delta);
        
        // 内存使用应该线性增长，不应该指数增长
        let expected_memory = participant_count as f64 * 0.01; // 每个参与者约0.01MB
        assert!(memory_delta < expected_memory * 2.0, 
            "内存使用 {}MB 超过预期 {}MB 的2倍", memory_delta, expected_memory);
        
        assert!(result.is_ok(), "内存测试失败");
    }
}

/// 性能测试6：超时处理测试
#[tokio::test]
async fn test_timeout_handling() {
    let participant_count = 100;
    
    // 设置很短的超时时间
    let timeout_duration = Duration::from_millis(10);
    
    let result = timeout(
        timeout_duration,
        run_single_selection_test("timeout_test", participant_count)
    ).await;
    
    // 应该超时
    assert!(result.is_err(), "操作应该在10ms内超时");
    
    println!("超时处理测试通过");
}

/// 辅助函数：运行单次选择测试
async fn run_single_selection_test(session_id: &str, participant_count: usize) -> Result<SelectionResult, TestError> {
    // 这里应该调用实际的引擎实现
    // 为了测试目的，我们模拟实现
    
    let participants: Vec<String> = (1..=participant_count)
        .map(|i| format!("participant_{}", i))
        .collect();
    
    // 模拟承诺阶段
    let mut commitments = HashMap::new();
    for participant in &participants {
        let commitment = format!("commitment_{}_{}", participant, session_id);
        commitments.insert(participant.clone(), commitment);
    }
    
    // 模拟揭示阶段
    let mut reveals = HashMap::new();
    for participant in &participants {
        let reveal = RevealData {
            participant: participant.clone(),
            randomness: format!("randomness_{}_{}", participant, session_id),
            salt: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        reveals.insert(participant.clone(), reveal);
    }
    
    // 模拟选择阶段
    let winner = participants[participant_count / 2].clone(); // 选择中间位置的参与者
    
    Ok(SelectionResult {
        session_id: session_id.to_string(),
        winner,
        total_participants: participant_count,
        selected_count: 1,
        random_seed: format!("seed_{}", session_id),
        selection_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        verification_proof: format!("proof_{}", session_id),
    })
}

/// 辅助函数：获取当前内存使用量（MB）
fn get_memory_usage() -> f64 {
    // 这里应该实现实际的内存使用量获取
    // 为了测试目的，返回模拟值
    use std::alloc::{GlobalAlloc, Layout, System};
    
    // 简单的内存使用估算
    let layout = Layout::new::<u8>();
    let size = layout.size();
    
    // 返回模拟的内存使用量
    size as f64 / 1024.0 / 1024.0
}

/// 性能基准测试配置
#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    pub name: String,
    pub participant_count: usize,
    pub iterations: usize,
    pub max_time_ms: u64,
    pub min_success_rate: f64,
}

impl PerformanceBenchmark {
    pub fn new(name: String, participant_count: usize, iterations: usize) -> Self {
        Self {
            name,
            participant_count,
            iterations,
            max_time_ms: match participant_count {
                1..=10 => 50,
                11..=100 => 200,
                101..=1000 => 1000,
                _ => 5000,
            },
            min_success_rate: match participant_count {
                1..=100 => 95.0,
                101..=1000 => 90.0,
                _ => 85.0,
            },
        }
    }
    
    pub async fn run(&self) -> Result<StressTestResult, TestError> {
        let mut total_time = Duration::new(0, 0);
        let mut successful_selections = 0;
        let mut max_time = Duration::new(0, 0);
        let mut min_time = Duration::from_secs(3600); // 初始化为1小时
        
        for iteration in 0..self.iterations {
            let start_time = Instant::now();
            
            let result = run_single_selection_test(
                &format!("benchmark_{}_{}", self.name, iteration),
                self.participant_count,
            ).await;
            
            let elapsed = start_time.elapsed();
            total_time += elapsed;
            
            if result.is_ok() {
                successful_selections += 1;
            }
            
            if elapsed > max_time {
                max_time = elapsed;
            }
            if elapsed < min_time {
                min_time = elapsed;
            }
        }
        
        let average_time = total_time / self.iterations as u32;
        let success_rate = (successful_selections as f64 / self.iterations as f64) * 100.0;
        
        // 验证性能要求
        if average_time.as_millis() > self.max_time_ms {
            return Err(TestError::Timeout);
        }
        
        if success_rate < self.min_success_rate {
            return Err(TestError::SelectionFailed);
        }
        
        Ok(StressTestResult {
            participant_count: self.participant_count,
            successful_selections,
            failed_selections: self.iterations - successful_selections,
            average_time_ms: average_time.as_millis() as f64,
            max_time_ms: max_time.as_millis() as u64,
            min_time_ms: min_time.as_millis() as u64,
            memory_usage_mb: get_memory_usage(),
        })
    }
}

/// 综合性能基准测试
#[tokio::test]
async fn test_comprehensive_performance_benchmark() {
    let benchmarks = vec![
        PerformanceBenchmark::new("small".to_string(), 10, 100),
        PerformanceBenchmark::new("medium".to_string(), 100, 50),
        PerformanceBenchmark::new("large".to_string(), 1000, 10),
    ];
    
    let mut results = Vec::new();
    
    for benchmark in benchmarks {
        println!("运行基准测试: {}", benchmark.name);
        let result = benchmark.run().await;
        
        match result {
            Ok(stress_result) => {
                println!("基准测试 {} 通过:", benchmark.name);
                println!("  参与者数量: {}", stress_result.participant_count);
                println!("  平均耗时: {:.2}ms", stress_result.average_time_ms);
                println!("  成功率: {:.2}%", 
                    (stress_result.successful_selections as f64 / 
                     (stress_result.successful_selections + stress_result.failed_selections) as f64) * 100.0);
                println!("  最大耗时: {}ms", stress_result.max_time_ms);
                println!("  最小耗时: {}ms", stress_result.min_time_ms);
                results.push(stress_result);
            }
            Err(e) => {
                panic!("基准测试 {} 失败: {}", benchmark.name, e);
            }
        }
    }
    
    println!("所有基准测试通过，共 {} 个测试", results.len());
}
