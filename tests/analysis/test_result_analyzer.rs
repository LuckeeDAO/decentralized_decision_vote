//! 测试结果分析工具
//! 
//! 用于分析单目标决策测试的结果，包括性能指标、随机性质量等

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 测试结果分析器
pub struct TestResultAnalyzer {
    results: Vec<TestResult>,
    config: AnalysisConfig,
}

/// 单个测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub participant_count: usize,
    pub execution_time_ms: u64,
    pub success: bool,
    pub winner: String,
    pub random_seed: String,
    pub memory_usage_mb: f64,
    pub timestamp: u64,
}

/// 分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub min_success_rate: f64,
    pub max_avg_time_ms: u64,
    pub min_randomness_quality: f64,
    pub max_memory_usage_mb: f64,
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub summary: TestSummary,
    pub performance_analysis: PerformanceAnalysis,
    pub randomness_analysis: RandomnessAnalysis,
    pub recommendations: Vec<String>,
    pub timestamp: u64,
}

/// 测试摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
}

/// 性能分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub time_by_scale: HashMap<String, TimeStats>,
    pub memory_usage_stats: MemoryStats,
    pub scalability_metrics: ScalabilityMetrics,
}

/// 时间统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStats {
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// 内存使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub min_mb: f64,
    pub max_mb: f64,
    pub avg_mb: f64,
    pub peak_mb: f64,
}

/// 可扩展性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityMetrics {
    pub time_complexity: String,
    pub memory_complexity: String,
    pub scalability_rating: f64,
}

/// 随机性分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomnessAnalysis {
    pub entropy_scores: HashMap<String, f64>,
    pub distribution_uniformity: HashMap<String, f64>,
    pub correlation_analysis: HashMap<String, f64>,
    pub overall_quality_score: f64,
}

impl TestResultAnalyzer {
    /// 创建新的分析器
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            results: Vec::new(),
            config,
        }
    }

    /// 添加测试结果
    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// 从文件加载结果
    pub fn load_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let results: Vec<TestResult> = serde_json::from_str(&content)?;
        self.results.extend(results);
        Ok(())
    }

    /// 生成分析报告
    pub fn generate_report(&self) -> AnalysisReport {
        let summary = self.calculate_summary();
        let performance_analysis = self.analyze_performance();
        let randomness_analysis = self.analyze_randomness();
        let recommendations = self.generate_recommendations(&summary, &performance_analysis, &randomness_analysis);

        AnalysisReport {
            summary,
            performance_analysis,
            randomness_analysis,
            recommendations,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// 计算测试摘要
    fn calculate_summary(&self) -> TestSummary {
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;
        let success_rate = if total_tests > 0 {
            (successful_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let total_execution_time_ms: u64 = self.results.iter().map(|r| r.execution_time_ms).sum();
        let average_execution_time_ms = if total_tests > 0 {
            total_execution_time_ms as f64 / total_tests as f64
        } else {
            0.0
        };

        TestSummary {
            total_tests,
            successful_tests,
            failed_tests,
            success_rate,
            total_execution_time_ms,
            average_execution_time_ms,
        }
    }

    /// 分析性能
    fn analyze_performance(&self) -> PerformanceAnalysis {
        let time_by_scale = self.calculate_time_by_scale();
        let memory_usage_stats = self.calculate_memory_stats();
        let scalability_metrics = self.calculate_scalability_metrics();

        PerformanceAnalysis {
            time_by_scale,
            memory_usage_stats,
            scalability_metrics,
        }
    }

    /// 按规模计算时间统计
    fn calculate_time_by_scale(&self) -> HashMap<String, TimeStats> {
        let mut scale_groups: HashMap<String, Vec<u64>> = HashMap::new();

        for result in &self.results {
            let scale = self.get_scale_category(result.participant_count);
            scale_groups.entry(scale).or_insert_with(Vec::new)
                .push(result.execution_time_ms);
        }

        let mut time_by_scale = HashMap::new();
        for (scale, times) in scale_groups {
            if !times.is_empty() {
                time_by_scale.insert(scale, self.calculate_time_stats(&times));
            }
        }

        time_by_scale
    }

    /// 获取规模类别
    fn get_scale_category(&self, participant_count: usize) -> String {
        match participant_count {
            1..=10 => "small".to_string(),
            11..=100 => "medium".to_string(),
            101..=1000 => "large".to_string(),
            _ => "xlarge".to_string(),
        }
    }

    /// 计算时间统计
    fn calculate_time_stats(&self, times: &[u64]) -> TimeStats {
        let mut sorted_times = times.to_vec();
        sorted_times.sort();

        let min_ms = *sorted_times.first().unwrap_or(&0);
        let max_ms = *sorted_times.last().unwrap_or(&0);
        let avg_ms = sorted_times.iter().sum::<u64>() as f64 / sorted_times.len() as f64;
        
        let median_ms = if sorted_times.len() % 2 == 0 {
            let mid = sorted_times.len() / 2;
            (sorted_times[mid - 1] + sorted_times[mid]) as f64 / 2.0
        } else {
            sorted_times[sorted_times.len() / 2] as f64
        };

        let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
        let p95_ms = sorted_times[p95_index.min(sorted_times.len() - 1)] as f64;

        let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
        let p99_ms = sorted_times[p99_index.min(sorted_times.len() - 1)] as f64;

        TimeStats {
            min_ms,
            max_ms,
            avg_ms,
            median_ms,
            p95_ms,
            p99_ms,
        }
    }

    /// 计算内存使用统计
    fn calculate_memory_stats(&self) -> MemoryStats {
        let memory_usages: Vec<f64> = self.results.iter().map(|r| r.memory_usage_mb).collect();
        
        if memory_usages.is_empty() {
            return MemoryStats {
                min_mb: 0.0,
                max_mb: 0.0,
                avg_mb: 0.0,
                peak_mb: 0.0,
            };
        }

        let min_mb = memory_usages.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_mb = memory_usages.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_mb = memory_usages.iter().sum::<f64>() / memory_usages.len() as f64;
        let peak_mb = max_mb;

        MemoryStats {
            min_mb,
            max_mb,
            avg_mb,
            peak_mb,
        }
    }

    /// 计算可扩展性指标
    fn calculate_scalability_metrics(&self) -> ScalabilityMetrics {
        // 分析时间复杂度和内存复杂度
        let time_complexity = self.analyze_time_complexity();
        let memory_complexity = self.analyze_memory_complexity();
        let scalability_rating = self.calculate_scalability_rating();

        ScalabilityMetrics {
            time_complexity,
            memory_complexity,
            scalability_rating,
        }
    }

    /// 分析时间复杂度
    fn analyze_time_complexity(&self) -> String {
        // 简化的复杂度分析
        let scales = vec![("small", 10), ("medium", 100), ("large", 1000)];
        let mut time_ratios = Vec::new();

        for (scale, expected_count) in scales {
            if let Some(times) = self.results.iter()
                .filter(|r| self.get_scale_category(r.participant_count) == scale)
                .map(|r| r.execution_time_ms)
                .collect::<Vec<_>>()
                .first() {
                time_ratios.push((scale, *times as f64 / expected_count as f64));
            }
        }

        if time_ratios.len() >= 2 {
            let ratio_growth = time_ratios[1].1 / time_ratios[0].1;
            if ratio_growth < 2.0 {
                "O(n)".to_string()
            } else if ratio_growth < 4.0 {
                "O(n log n)".to_string()
            } else {
                "O(n²)".to_string()
            }
        } else {
            "Unknown".to_string()
        }
    }

    /// 分析内存复杂度
    fn analyze_memory_complexity(&self) -> String {
        // 简化的内存复杂度分析
        let avg_memory_per_participant = self.results.iter()
            .map(|r| r.memory_usage_mb / r.participant_count as f64)
            .sum::<f64>() / self.results.len() as f64;

        if avg_memory_per_participant < 0.01 {
            "O(1)".to_string()
        } else if avg_memory_per_participant < 0.1 {
            "O(n)".to_string()
        } else {
            "O(n²)".to_string()
        }
    }

    /// 计算可扩展性评分
    fn calculate_scalability_rating(&self) -> f64 {
        let time_complexity_score = match self.analyze_time_complexity().as_str() {
            "O(1)" => 1.0,
            "O(n)" => 0.9,
            "O(n log n)" => 0.7,
            "O(n²)" => 0.3,
            _ => 0.5,
        };

        let memory_complexity_score = match self.analyze_memory_complexity().as_str() {
            "O(1)" => 1.0,
            "O(n)" => 0.9,
            "O(n²)" => 0.3,
            _ => 0.5,
        };

        (time_complexity_score + memory_complexity_score) / 2.0
    }

    /// 分析随机性
    fn analyze_randomness(&self) -> RandomnessAnalysis {
        let entropy_scores = self.calculate_entropy_scores();
        let distribution_uniformity = self.calculate_distribution_uniformity();
        let correlation_analysis = self.calculate_correlation_analysis();
        let overall_quality_score = self.calculate_overall_quality_score(&entropy_scores, &distribution_uniformity);

        RandomnessAnalysis {
            entropy_scores,
            distribution_uniformity,
            correlation_analysis,
            overall_quality_score,
        }
    }

    /// 计算熵分数
    fn calculate_entropy_scores(&self) -> HashMap<String, f64> {
        let mut entropy_scores = HashMap::new();
        
        for result in &self.results {
            let scale = self.get_scale_category(result.participant_count);
            let entropy = self.calculate_entropy(&result.random_seed);
            entropy_scores.insert(scale, entropy);
        }

        entropy_scores
    }

    /// 计算字符串的熵
    fn calculate_entropy(&self, s: &str) -> f64 {
        let mut counts = HashMap::new();
        for ch in s.chars() {
            *counts.entry(ch).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        counts.values().map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        }).sum()
    }

    /// 计算分布均匀性
    fn calculate_distribution_uniformity(&self) -> HashMap<String, f64> {
        let mut distribution_uniformity = HashMap::new();
        
        // 按规模分组分析获胜者分布
        let mut scale_groups: HashMap<String, Vec<String>> = HashMap::new();
        for result in &self.results {
            let scale = self.get_scale_category(result.participant_count);
            scale_groups.entry(scale).or_insert_with(Vec::new)
                .push(result.winner.clone());
        }

        for (scale, winners) in scale_groups {
            let uniformity = self.calculate_winner_distribution_uniformity(&winners);
            distribution_uniformity.insert(scale, uniformity);
        }

        distribution_uniformity
    }

    /// 计算获胜者分布均匀性
    fn calculate_winner_distribution_uniformity(&self, winners: &[String]) -> f64 {
        let mut counts = HashMap::new();
        for winner in winners {
            *counts.entry(winner.clone()).or_insert(0) += 1;
        }

        if counts.is_empty() {
            return 0.0;
        }

        let total = winners.len() as f64;
        let expected = total / counts.len() as f64;
        
        let variance = counts.values().map(|&count| {
            let diff = count as f64 - expected;
            diff * diff
        }).sum::<f64>() / counts.len() as f64;

        // 转换为0-1之间的均匀性分数
        let max_variance = expected * expected;
        1.0 - (variance / max_variance).min(1.0)
    }

    /// 计算相关性分析
    fn calculate_correlation_analysis(&self) -> HashMap<String, f64> {
        // 简化的相关性分析
        let mut correlation_analysis = HashMap::new();
        
        for result in &self.results {
            let scale = self.get_scale_category(result.participant_count);
            // 这里应该实现更复杂的相关性分析
            // 目前返回一个模拟值
            correlation_analysis.insert(scale, 0.1);
        }

        correlation_analysis
    }

    /// 计算整体质量分数
    fn calculate_overall_quality_score(&self, entropy_scores: &HashMap<String, f64>, distribution_uniformity: &HashMap<String, f64>) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        for scale in entropy_scores.keys() {
            if let (Some(&entropy), Some(&uniformity)) = (entropy_scores.get(scale), distribution_uniformity.get(scale)) {
                total_score += (entropy + uniformity) / 2.0;
                count += 1;
            }
        }

        if count > 0 {
            total_score / count as f64
        } else {
            0.0
        }
    }

    /// 生成建议
    fn generate_recommendations(&self, summary: &TestSummary, performance: &PerformanceAnalysis, randomness: &RandomnessAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 成功率建议
        if summary.success_rate < self.config.min_success_rate {
            recommendations.push(format!(
                "成功率 {:.1}% 低于要求 {:.1}%，需要优化错误处理机制",
                summary.success_rate, self.config.min_success_rate
            ));
        }

        // 性能建议
        if summary.average_execution_time_ms > self.config.max_avg_time_ms as f64 {
            recommendations.push(format!(
                "平均执行时间 {:.1}ms 超过要求 {}ms，需要性能优化",
                summary.average_execution_time_ms, self.config.max_avg_time_ms
            ));
        }

        // 可扩展性建议
        if performance.scalability_metrics.scalability_rating < 0.7 {
            recommendations.push(format!(
                "可扩展性评分 {:.2} 较低，建议优化算法复杂度",
                performance.scalability_metrics.scalability_rating
            ));
        }

        // 随机性建议
        if randomness.overall_quality_score < self.config.min_randomness_quality {
            recommendations.push(format!(
                "随机性质量评分 {:.2} 低于要求 {:.2}，需要改进随机数生成算法",
                randomness.overall_quality_score, self.config.min_randomness_quality
            ));
        }

        // 内存使用建议
        if performance.memory_usage_stats.peak_mb > self.config.max_memory_usage_mb {
            recommendations.push(format!(
                "峰值内存使用 {:.1}MB 超过限制 {:.1}MB，需要内存优化",
                performance.memory_usage_stats.peak_mb, self.config.max_memory_usage_mb
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("所有指标均符合要求，系统运行良好".to_string());
        }

        recommendations
    }

    /// 保存报告到文件
    pub fn save_report<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.generate_report();
        let content = serde_json::to_string_pretty(&report)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    /// 生成HTML报告
    pub fn generate_html_report(&self) -> String {
        let report = self.generate_report();
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>单目标决策测试分析报告</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .section {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .metric {{ display: inline-block; margin: 10px; padding: 10px; background-color: #e8f4f8; border-radius: 3px; }}
        .recommendation {{ background-color: #fff3cd; padding: 10px; margin: 5px 0; border-left: 4px solid #ffc107; }}
        .success {{ color: #28a745; }}
        .warning {{ color: #ffc107; }}
        .error {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>单目标决策测试分析报告</h1>
        <p>生成时间: {}</p>
    </div>

    <div class="section">
        <h2>测试摘要</h2>
        <div class="metric">总测试数: {}</div>
        <div class="metric">成功测试: {}</div>
        <div class="metric">失败测试: {}</div>
        <div class="metric">成功率: {:.1}%</div>
        <div class="metric">平均执行时间: {:.1}ms</div>
    </div>

    <div class="section">
        <h2>性能分析</h2>
        <h3>可扩展性指标</h3>
        <div class="metric">时间复杂度: {}</div>
        <div class="metric">内存复杂度: {}</div>
        <div class="metric">可扩展性评分: {:.2}</div>
    </div>

    <div class="section">
        <h2>随机性分析</h2>
        <div class="metric">整体质量评分: {:.2}</div>
    </div>

    <div class="section">
        <h2>建议</h2>
        {}
    </div>
</body>
</html>
        "#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        report.summary.total_tests,
        report.summary.successful_tests,
        report.summary.failed_tests,
        report.summary.success_rate,
        report.summary.average_execution_time_ms,
        report.performance_analysis.scalability_metrics.time_complexity,
        report.performance_analysis.scalability_metrics.memory_complexity,
        report.performance_analysis.scalability_metrics.scalability_rating,
        report.randomness_analysis.overall_quality_score,
        report.recommendations.iter()
            .map(|rec| format!("<div class=\"recommendation\">{}</div>", rec))
            .collect::<Vec<_>>()
            .join("")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let config = AnalysisConfig {
            min_success_rate: 95.0,
            max_avg_time_ms: 100,
            min_randomness_quality: 0.8,
            max_memory_usage_mb: 100.0,
        };
        
        let analyzer = TestResultAnalyzer::new(config);
        assert_eq!(analyzer.results.len(), 0);
    }

    #[test]
    fn test_result_analysis() {
        let config = AnalysisConfig {
            min_success_rate: 95.0,
            max_avg_time_ms: 100,
            min_randomness_quality: 0.8,
            max_memory_usage_mb: 100.0,
        };
        
        let mut analyzer = TestResultAnalyzer::new(config);
        
        // 添加测试结果
        analyzer.add_result(TestResult {
            test_name: "test_1".to_string(),
            participant_count: 10,
            execution_time_ms: 50,
            success: true,
            winner: "alice".to_string(),
            random_seed: "seed123".to_string(),
            memory_usage_mb: 10.0,
            timestamp: 1234567890,
        });
        
        let report = analyzer.generate_report();
        assert_eq!(report.summary.total_tests, 1);
        assert_eq!(report.summary.successful_tests, 1);
        assert_eq!(report.summary.success_rate, 100.0);
    }
}
