//! 新区块链功能展示示例
//!
//! 展示 Archway、Injective、Avalanche、Sui 等新区块链的存储功能

use blockchain_store::{
    BlockchainManager, BlockchainConfig, BlockchainType, 
    StorageTransaction, Result
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let config = BlockchainConfig::from_file("examples/config.json")
        .map_err(|e| blockchain_store::BlockchainError::InvalidConfig(e.to_string()))?;

    // 创建区块链管理器
    let mut manager = BlockchainManager::new(config);
    manager.initialize().await?;

    println!("=== 新区块链存储功能展示 ===");

    // 测试数据
    let test_data = b"New Blockchain Storage Test Data";
    let test_key = "new_blockchain_test";

    // 1. Archway 存储演示
    println!("\n1. Archway 区块链存储演示");
    println!("   - 特性: 开发者奖励、Cosmos 生态、低费用");
    
    let archway_metadata = json!({
        "blockchain": "Archway",
        "features": ["developer_rewards", "cosmos_ecosystem", "low_fees"],
        "description": "Archway 智能合约存储测试"
    });

    let archway_tx = manager.store_data(
        &BlockchainType::Archway,
        &format!("{}_archway", test_key),
        test_data,
        Some(archway_metadata)
    ).await?;

    println!("   ✅ 存储成功:");
    println!("     交易哈希: {}", archway_tx.tx_hash);
    println!("     区块号: {:?}", archway_tx.block_number);
    println!("     Gas 使用量: {:?}", archway_tx.gas_used);
    println!("     状态: {:?}", archway_tx.status);

    // 2. Injective 存储演示
    println!("\n2. Injective 区块链存储演示");
    println!("   - 特性: 去中心化交易、跨链支持、高性能");
    
    let injective_metadata = json!({
        "blockchain": "Injective",
        "features": ["decentralized_trading", "cross_chain", "high_performance"],
        "description": "Injective 去中心化交易数据存储测试"
    });

    let injective_tx = manager.store_data(
        &BlockchainType::Injective,
        &format!("{}_injective", test_key),
        test_data,
        Some(injective_metadata)
    ).await?;

    println!("   ✅ 存储成功:");
    println!("     交易哈希: {}", injective_tx.tx_hash);
    println!("     区块号: {:?}", injective_tx.block_number);
    println!("     Gas 使用量: {:?}", injective_tx.gas_used);
    println!("     状态: {:?}", injective_tx.status);

    // 3. Avalanche 存储演示
    println!("\n3. Avalanche 区块链存储演示");
    println!("   - 特性: 高吞吐量、以太坊兼容、快速确认");
    
    let avalanche_metadata = json!({
        "blockchain": "Avalanche",
        "features": ["high_throughput", "ethereum_compatible", "fast_confirmation"],
        "description": "Avalanche 高吞吐量存储测试"
    });

    let avalanche_tx = manager.store_data(
        &BlockchainType::Avalanche,
        &format!("{}_avalanche", test_key),
        test_data,
        Some(avalanche_metadata)
    ).await?;

    println!("   ✅ 存储成功:");
    println!("     交易哈希: {}", avalanche_tx.tx_hash);
    println!("     区块号: {:?}", avalanche_tx.block_number);
    println!("     Gas 使用量: {:?}", avalanche_tx.gas_used);
    println!("     状态: {:?}", avalanche_tx.status);

    // 4. Sui 存储演示
    println!("\n4. Sui 区块链存储演示");
    println!("   - 特性: 对象存储、并行执行、Move 语言");
    
    let sui_metadata = json!({
        "blockchain": "Sui",
        "features": ["object_storage", "parallel_execution", "move_language"],
        "description": "Sui 对象存储测试"
    });

    let sui_tx = manager.store_data(
        &BlockchainType::Sui,
        &format!("{}_sui", test_key),
        test_data,
        Some(sui_metadata)
    ).await?;

    println!("   ✅ 存储成功:");
    println!("     交易哈希: {}", sui_tx.tx_hash);
    println!("     Checkpoint: {:?}", sui_tx.block_number);
    println!("     Gas Units: {:?}", sui_tx.gas_used);
    println!("     状态: {:?}", sui_tx.status);

    // 5. 数据检索测试
    println!("\n5. 数据检索测试");
    
    let blockchains = [
        (BlockchainType::Archway, "Archway"),
        (BlockchainType::Injective, "Injective"),
        (BlockchainType::Avalanche, "Avalanche"),
        (BlockchainType::Sui, "Sui"),
    ];

    for (blockchain_type, name) in blockchains {
        let key = format!("{}_{}", test_key, name.to_lowercase());
        match manager.retrieve_data(&blockchain_type, &key).await {
            Ok(data) => println!("   ✅ {} 检索成功: {} bytes", name, data.len()),
            Err(e) => println!("   ❌ {} 检索失败: {}", name, e),
        }
    }

    // 6. 性能对比测试
    println!("\n6. 性能对比测试");
    
    let performance_data = b"Performance test data for blockchain comparison";
    
    for (blockchain_type, name) in blockchains {
        let start = std::time::Instant::now();
        
        match manager.store_data(
            &blockchain_type,
            &format!("perf_test_{}", name.to_lowercase()),
            performance_data,
            None
        ).await {
            Ok(tx) => {
                let duration = start.elapsed();
                println!("   {} 存储性能:", name);
                println!("     耗时: {:?}", duration);
                println!("     Gas 使用量: {:?}", tx.gas_used);
                println!("     交易哈希: {}", tx.tx_hash);
            }
            Err(e) => println!("   {} 存储失败: {}", name, e),
        }
    }

    // 7. 数据完整性验证
    println!("\n7. 数据完整性验证");
    
    let expected_hash = hex::encode(sha2::Sha256::digest(test_data));
    
    for (blockchain_type, name) in blockchains {
        let key = format!("{}_{}", test_key, name.to_lowercase());
        match manager.verify_data(&blockchain_type, &key, &expected_hash).await {
            Ok(is_valid) => {
                let status = if is_valid { "✅ 通过" } else { "❌ 失败" };
                println!("   {} 数据完整性验证: {}", name, status);
            }
            Err(e) => println!("   {} 数据完整性验证失败: {}", name, e),
        }
    }

    // 8. 统计信息展示
    println!("\n8. 统计信息展示");
    
    let all_stats = manager.get_all_stats().await?;
    
    for (blockchain_type, name) in blockchains {
        if let Some(stats) = all_stats.get(&blockchain_type) {
            println!("   {} 统计信息:", name);
            println!("     总交易数: {}", stats.total_transactions);
            println!("     总数据大小: {} bytes", stats.total_data_size);
            println!("     平均 Gas 使用量: {:.2}", stats.average_gas_used);
            println!("     成功率: {:.2}%", stats.success_rate * 100.0);
            println!("     最后更新: {}", stats.last_updated.format("%Y-%m-%d %H:%M:%S"));
        }
    }

    // 9. 区块链特性总结
    println!("\n9. 区块链特性总结");
    println!("   📊 支持的区块链数量: {}", blockchains.len());
    println!("   🔗 跨链存储能力: 支持多链数据存储");
    println!("   ⚡ 性能优化: 各链针对不同场景优化");
    println!("   🛡️ 安全性: 多重验证和完整性检查");
    println!("   🔧 可扩展性: 易于添加新的区块链支持");

    println!("\n=== 新区块链功能展示完成 ===");
    Ok(())
}
