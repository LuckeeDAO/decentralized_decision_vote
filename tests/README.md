# 单目标决策（n选1）测试套件

本测试套件专门用于测试基于比特承诺模型的去中心化随机数生成系统，实现从n个参与者中选出1个中奖人的功能。

## 测试架构说明

本测试套件支持两种服务端架构：
- **单体服务端** (`service/`)：集成测试和端到端测试
- **微服务架构** (`services/`)：分布式系统测试

## 测试架构

### 测试层次结构

```
tests/
├── unit/                    # 单元测试
├── integration/            # 集成测试
│   ├── single_target_selection_tests.rs  # 单目标选择核心测试
│   ├── test_types.rs       # 测试数据类型定义
│   └── common.rs          # 通用测试工具
├── performance/            # 性能测试
│   └── single_target_performance_tests.rs
├── e2e/                   # 端到端测试
│   └── single_target_e2e_tests.rs
├── analysis/              # 测试结果分析
│   └── test_result_analyzer.rs
├── test_config.toml       # 测试配置
├── run_single_target_tests.sh  # 测试运行脚本
└── README.md              # 本文档
```

## 测试用例分类

### 1. 基础功能测试

#### 1.1 小规模测试（3-10个参与者）
- **test_basic_single_target_selection_3_choose_1**: 3选1基础测试
- **test_medium_single_target_selection_10_choose_1**: 10选1中等规模测试
- **test_single_participant**: 单参与者边界情况测试

#### 1.2 中等规模测试（50-100个参与者）
- **test_medium_single_target_selection_10_choose_1**: 10选1测试
- **test_large_single_target_selection_100_choose_1**: 100选1大规模测试

#### 1.3 大规模测试（500-1000个参与者）
- **test_large_single_target_selection_100_choose_1**: 100选1测试
- 可扩展到更大规模

### 2. 随机性质量测试

#### 2.1 分布均匀性测试
- **test_randomness_distribution**: 验证多次运行的结果分布
- 要求：每个参与者获胜概率接近1/n
- 允许偏差：±50%以内

#### 2.2 熵分析测试
- 验证随机种子的熵值
- 要求：熵值 > 0.8

#### 2.3 相关性分析
- 验证不同轮次之间的独立性
- 要求：相关系数 < 0.1

### 3. 安全性测试

#### 3.1 承诺验证测试
- **test_commitment_verification_failure**: 承诺不匹配检测
- 验证承诺-揭示机制的正确性

#### 3.2 超时处理测试
- **test_timeout_scenarios**: 超时场景处理
- 验证时间窗口限制

#### 3.3 异常输入测试
- 无效参与者处理
- 恶意数据检测

### 4. 性能测试

#### 4.1 响应时间测试
- **小规模（≤10人）**: 平均耗时 < 50ms
- **中等规模（11-100人）**: 平均耗时 < 200ms
- **大规模（101-1000人）**: 平均耗时 < 1000ms

#### 4.2 并发性能测试
- **test_concurrent_performance**: 并发会话处理
- 要求：10个并发会话，总耗时 < 5秒

#### 4.3 内存使用测试
- **test_memory_usage**: 内存使用分析
- 要求：线性增长，每个参与者 < 0.01MB

#### 4.4 可扩展性测试
- 时间复杂度分析
- 内存复杂度分析
- 可扩展性评分 > 0.7

### 5. 端到端测试

#### 5.1 完整流程测试
- **test_complete_lottery_flow_5_choose_1**: 5选1完整流程
- **test_large_scale_lottery_flow_50_choose_1**: 50选1大规模流程

#### 5.2 数据持久化测试
- **test_data_persistence**: 数据保存和恢复
- 验证系统重启后的数据完整性

#### 5.3 并发访问测试
- **test_concurrent_access**: 多用户并发访问
- 验证数据一致性

#### 5.4 错误恢复测试
- **test_error_handling_scenarios**: 异常情况处理
- 验证系统鲁棒性

## 测试配置

### 性能要求

| 规模 | 参与者数量 | 最大耗时 | 最小成功率 | 内存限制 |
|------|------------|----------|------------|----------|
| 小规模 | 1-10 | 50ms | 95% | 10MB |
| 中等规模 | 11-100 | 200ms | 95% | 50MB |
| 大规模 | 101-1000 | 1000ms | 90% | 200MB |
| 超大规模 | 1001+ | 5000ms | 85% | 500MB |

### 随机性要求

- **熵分数**: > 0.8
- **分布均匀性**: > 0.9
- **最大相关性**: < 0.1

## 运行测试

### 运行所有测试
```bash
# 单体服务端测试
cd service
cargo test

# 微服务架构测试
cd services/vote-api
cargo test

# 集成测试
./tests/run_single_target_tests.sh
```

### 运行特定测试类别
```bash
# 基础功能测试
cargo test --test single_target_selection_tests

# 性能测试
cargo test --test single_target_performance_tests

# 端到端测试
cargo test --test single_target_e2e_tests

# 微服务测试
cd services/vote-api
cargo test --test integration_tests
```

### 运行特定测试
```bash
# 运行3选1测试
cargo test test_basic_single_target_selection_3_choose_1

# 运行性能基准测试
cargo test test_comprehensive_performance_benchmark
```

## 测试结果分析

### 自动分析
测试完成后会自动生成分析报告：
- JSON格式的详细报告
- HTML格式的可视化报告
- 性能指标统计
- 随机性质量评估

### 关键指标

1. **成功率**: 所有测试的通过率
2. **平均响应时间**: 按规模分组的平均耗时
3. **可扩展性评分**: 基于时间和内存复杂度的综合评分
4. **随机性质量**: 基于熵和分布均匀性的评分

### 报告示例

```json
{
  "summary": {
    "total_tests": 25,
    "successful_tests": 24,
    "success_rate": 96.0,
    "average_execution_time_ms": 45.2
  },
  "performance_analysis": {
    "scalability_metrics": {
      "time_complexity": "O(n)",
      "memory_complexity": "O(n)",
      "scalability_rating": 0.9
    }
  },
  "randomness_analysis": {
    "overall_quality_score": 0.92
  }
}
```

## 测试数据管理

### 测试数据目录
```
test_data/
├── session_*.json          # 会话配置
├── *_commitments.json      # 承诺数据
├── *_reveals.json         # 揭示数据
├── *_result.json          # 选择结果
└── test_results/          # 测试报告
    ├── test.log           # 测试日志
    ├── report.json        # JSON报告
    └── report.html        # HTML报告
```

### 数据清理
- 测试完成后自动清理临时数据
- 保留重要的测试报告和日志
- 支持手动清理：`rm -rf ./test_data`

## 持续集成

### CI/CD集成
测试套件设计为支持持续集成：
- 并行测试执行
- 测试结果缓存
- 失败重试机制
- 性能回归检测

### 测试环境要求
- Rust 1.70+
- 内存: 至少1GB可用内存
- 磁盘: 至少100MB可用空间
- 网络: 可选（用于外部依赖）

## 故障排除

### 常见问题

1. **测试超时**
   - 检查系统资源使用情况
   - 调整超时配置
   - 减少并发测试数量

2. **内存不足**
   - 减少大规模测试的参与者数量
   - 增加系统内存
   - 优化测试数据管理

3. **随机性测试失败**
   - 检查随机数生成算法
   - 增加测试迭代次数
   - 调整随机性质量阈值

### 调试模式
```bash
# 启用详细日志
RUST_LOG=debug cargo test

# 启用跟踪
RUST_BACKTRACE=1 cargo test

# 单线程运行（避免并发问题）
cargo test -- --test-threads=1
```

## 扩展测试

### 添加新测试用例
1. 在相应的测试文件中添加新函数
2. 使用 `#[tokio::test]` 标记异步测试
3. 遵循命名约定：`test_<功能>_<场景>`
4. 添加适当的断言和验证

### 自定义测试配置
修改 `test_config.toml` 文件：
- 调整性能要求
- 修改测试规模
- 配置分析参数

### 集成新算法
1. 在 `SelectionAlgorithm` 枚举中添加新类型
2. 实现相应的选择逻辑
3. 添加专门的测试用例
4. 更新性能基准

## 贡献指南

### 测试代码规范
- 使用清晰的测试名称
- 添加充分的注释
- 遵循Rust测试最佳实践
- 确保测试的可重复性

### 提交测试
1. 确保所有测试通过
2. 更新相关文档
3. 添加适当的测试用例
4. 验证性能要求

---

**注意**: 本测试套件专注于单目标决策（n选1）场景。对于多目标决策（n选k）和多等级决策的测试，请参考相应的测试文档。
