#!/bin/bash

# 单目标决策（n选1）测试运行脚本
# 用于运行所有相关的测试用例

set -e

echo "=========================================="
echo "单目标决策（n选1）测试套件"
echo "=========================================="

# 创建测试数据目录
mkdir -p ./test_data

# 设置环境变量
export RUST_LOG=info
export RUST_BACKTRACE=1

echo "开始运行测试..."

# 1. 运行单元测试
echo ""
echo "1. 运行单元测试..."
cargo test --package decentralized_decision_vote --lib -- --nocapture

# 2. 运行集成测试
echo ""
echo "2. 运行集成测试..."
cargo test --package decentralized_decision_vote --test single_target_selection_tests -- --nocapture

# 3. 运行性能测试
echo ""
echo "3. 运行性能测试..."
cargo test --package decentralized_decision_vote --test single_target_performance_tests -- --nocapture

# 4. 运行端到端测试
echo ""
echo "4. 运行端到端测试..."
cargo test --package decentralized_decision_vote --test single_target_e2e_tests -- --nocapture

# 5. 运行所有测试（确保没有遗漏）
echo ""
echo "5. 运行所有测试..."
cargo test --package decentralized_decision_vote -- --nocapture

echo ""
echo "=========================================="
echo "所有测试完成！"
echo "=========================================="

# 清理测试数据
echo "清理测试数据..."
rm -rf ./test_data

echo "测试套件执行完毕。"
