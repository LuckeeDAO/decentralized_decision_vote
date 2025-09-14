//! 区块链存储使用示例

use blockchain_store::{
    BlockchainManager, BlockchainConfig, BlockchainType, 
    StorageTransaction, StorageMetadata, Result
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
    
    // 初始化所有区块链客户端
    manager.initialize().await?;

    // 示例数据
    let test_data = b"Hello, Blockchain Storage!";
    let test_key = "test_key_001";
    let test_metadata = json!({
        "description": "Test data for blockchain storage",
        "version": "1.0",
        "tags": ["test", "example"]
    });

    println!("=== 区块链存储示例 ===");

    // 1. 存储数据到以太坊
    println!("\n1. 存储数据到以太坊...");
    let eth_tx = manager.store_data(
        &BlockchainType::Ethereum,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("以太坊交易哈希: {}", eth_tx.tx_hash);
    println!("区块号: {:?}", eth_tx.block_number);
    println!("Gas 使用量: {:?}", eth_tx.gas_used);
    println!("状态: {:?}", eth_tx.status);

    // 2. 存储数据到 Solana
    println!("\n2. 存储数据到 Solana...");
    let sol_tx = manager.store_data(
        &BlockchainType::Solana,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Solana 交易哈希: {}", sol_tx.tx_hash);
    println!("Slot: {:?}", sol_tx.block_number);
    println!("Compute Units: {:?}", sol_tx.gas_used);
    println!("状态: {:?}", sol_tx.status);

    // 3. 存储数据到 Cosmos
    println!("\n3. 存储数据到 Cosmos...");
    let cosmos_tx = manager.store_data(
        &BlockchainType::Cosmos,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Cosmos 交易哈希: {}", cosmos_tx.tx_hash);
    println!("区块号: {:?}", cosmos_tx.block_number);
    println!("Gas 使用量: {:?}", cosmos_tx.gas_used);
    println!("状态: {:?}", cosmos_tx.status);

    // 4. 存储数据到 Archway
    println!("\n4. 存储数据到 Archway...");
    let archway_tx = manager.store_data(
        &BlockchainType::Archway,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Archway 交易哈希: {}", archway_tx.tx_hash);
    println!("区块号: {:?}", archway_tx.block_number);
    println!("Gas 使用量: {:?}", archway_tx.gas_used);
    println!("状态: {:?}", archway_tx.status);

    // 5. 存储数据到 Injective
    println!("\n5. 存储数据到 Injective...");
    let injective_tx = manager.store_data(
        &BlockchainType::Injective,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Injective 交易哈希: {}", injective_tx.tx_hash);
    println!("区块号: {:?}", injective_tx.block_number);
    println!("Gas 使用量: {:?}", injective_tx.gas_used);
    println!("状态: {:?}", injective_tx.status);

    // 6. 存储数据到 Avalanche
    println!("\n6. 存储数据到 Avalanche...");
    let avalanche_tx = manager.store_data(
        &BlockchainType::Avalanche,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Avalanche 交易哈希: {}", avalanche_tx.tx_hash);
    println!("区块号: {:?}", avalanche_tx.block_number);
    println!("Gas 使用量: {:?}", avalanche_tx.gas_used);
    println!("状态: {:?}", avalanche_tx.status);

    // 7. 存储数据到 Sui
    println!("\n7. 存储数据到 Sui...");
    let sui_tx = manager.store_data(
        &BlockchainType::Sui,
        test_key,
        test_data,
        Some(test_metadata.clone())
    ).await?;
    
    println!("Sui 交易哈希: {}", sui_tx.tx_hash);
    println!("Checkpoint: {:?}", sui_tx.block_number);
    println!("Gas Units: {:?}", sui_tx.gas_used);
    println!("状态: {:?}", sui_tx.status);

    // 8. 从不同区块链检索数据
    println!("\n8. 从不同区块链检索数据...");
    
    // 从以太坊检索
    match manager.retrieve_data(&BlockchainType::Ethereum, test_key).await {
        Ok(data) => println!("从以太坊检索到数据: {} bytes", data.len()),
        Err(e) => println!("从以太坊检索失败: {}", e),
    }

    // 从 Solana 检索
    match manager.retrieve_data(&BlockchainType::Solana, test_key).await {
        Ok(data) => println!("从 Solana 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Solana 检索失败: {}", e),
    }

    // 从 Cosmos 检索
    match manager.retrieve_data(&BlockchainType::Cosmos, test_key).await {
        Ok(data) => println!("从 Cosmos 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Cosmos 检索失败: {}", e),
    }

    // 从 Archway 检索
    match manager.retrieve_data(&BlockchainType::Archway, test_key).await {
        Ok(data) => println!("从 Archway 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Archway 检索失败: {}", e),
    }

    // 从 Injective 检索
    match manager.retrieve_data(&BlockchainType::Injective, test_key).await {
        Ok(data) => println!("从 Injective 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Injective 检索失败: {}", e),
    }

    // 从 Avalanche 检索
    match manager.retrieve_data(&BlockchainType::Avalanche, test_key).await {
        Ok(data) => println!("从 Avalanche 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Avalanche 检索失败: {}", e),
    }

    // 从 Sui 检索
    match manager.retrieve_data(&BlockchainType::Sui, test_key).await {
        Ok(data) => println!("从 Sui 检索到数据: {} bytes", data.len()),
        Err(e) => println!("从 Sui 检索失败: {}", e),
    }

    // 5. 获取统计信息
    println!("\n5. 获取统计信息...");
    let all_stats = manager.get_all_stats().await?;
    
    for (blockchain_type, stats) in all_stats {
        println!("\n{:?} 统计信息:", blockchain_type);
        println!("  总交易数: {}", stats.total_transactions);
        println!("  总数据大小: {} bytes", stats.total_data_size);
        println!("  平均 Gas 使用量: {:.2}", stats.average_gas_used);
        println!("  成功率: {:.2}%", stats.success_rate * 100.0);
        println!("  最后更新: {}", stats.last_updated);
    }

    // 9. 验证数据完整性
    println!("\n9. 验证数据完整性...");
    let expected_hash = hex::encode(sha2::Sha256::digest(test_data));
    
    for blockchain_type in [
        BlockchainType::Ethereum, 
        BlockchainType::Solana, 
        BlockchainType::Cosmos,
        BlockchainType::Archway,
        BlockchainType::Injective,
        BlockchainType::Avalanche,
        BlockchainType::Sui
    ] {
        match manager.verify_data(&blockchain_type, test_key, &expected_hash).await {
            Ok(is_valid) => println!("{:?} 数据完整性验证: {}", blockchain_type, if is_valid { "通过" } else { "失败" }),
            Err(e) => println!("{:?} 数据完整性验证失败: {}", blockchain_type, e),
        }
    }

    println!("\n=== 示例完成 ===");
    Ok(())
}
