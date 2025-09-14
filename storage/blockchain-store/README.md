# 区块链存储模块 (Blockchain Store)

## 概述

区块链存储模块是一个独立的 Rust crate，为 Luckee DAO 去中心化决策投票系统提供统一的区块链存储抽象层。该模块支持多种主流区块链网络，包括sui、以太坊、Solana、Cosmos 等，通过参数配置可以轻松切换不同的区块链访问方式。

## 核心特性

- **多区块链支持**: 支持以太坊、Solana、Cosmos 等主流区块链
- **统一接口**: 提供一致的 API 接口，屏蔽不同区块链的差异
- **配置驱动**: 通过配置文件管理不同区块链网络的连接参数
- **异步操作**: 全面支持异步操作，提高性能
- **错误处理**: 完善的错误处理和重试机制
- **数据验证**: 内置数据完整性验证功能
- **统计监控**: 提供详细的存储统计信息

## 支持的区块链

| 区块链 | 状态 | 特性 |
|--------|------|------|
| Ethereum | ✅ 支持 | 智能合约存储、Gas 优化 |
| Solana | ✅ 支持 | 账户存储、低费用 |
| Cosmos | ✅ 支持 | 智能合约存储、IBC 支持 |
| Archway | ✅ 支持 | 智能合约存储、开发者奖励 |
| Injective | ✅ 支持 | 去中心化交易、跨链支持 |
| Avalanche | ✅ 支持 | 以太坊兼容、高吞吐量 |
| Sui | ✅ 支持 | 对象存储、并行执行 |
| Polygon | 🔄 开发中 | 以太坊兼容、低费用 |
| Arbitrum | 🔄 开发中 | Layer 2 扩容 |
| Optimism | 🔄 开发中 | Layer 2 扩容 |
| BSC | 🔄 开发中 | 以太坊兼容 |

## 快速开始

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
blockchain-store = { path = "storage/blockchain-store" }
```

### 2. 创建配置文件

创建 `config.json` 文件：

```json
{
  "default_blockchain": "Ethereum",
  "networks": {
    "ethereum_mainnet": {
      "name": "Ethereum Mainnet",
      "rpc_url": "https://mainnet.infura.io/v3/YOUR_PROJECT_ID",
      "chain_id": 1,
      "gas_price": "20",
      "gas_limit": 21000,
      "timeout_seconds": 30,
      "retry_attempts": 3
    },
    "solana_mainnet": {
      "name": "Solana Mainnet",
      "rpc_url": "https://api.mainnet-beta.solana.com",
      "timeout_seconds": 30,
      "retry_attempts": 3
    },
    "archway_mainnet": {
      "name": "Archway Mainnet",
      "rpc_url": "https://rpc.mainnet.archway.io",
      "gas_price": "0.02",
      "gas_limit": 150000,
      "timeout_seconds": 30,
      "retry_attempts": 3
    },
    "injective_mainnet": {
      "name": "Injective Mainnet",
      "rpc_url": "https://tm.injective.network",
      "gas_price": "0.03",
      "gas_limit": 180000,
      "timeout_seconds": 30,
      "retry_attempts": 3
    },
    "avalanche_mainnet": {
      "name": "Avalanche C-Chain",
      "rpc_url": "https://api.avax.network/ext/bc/C/rpc",
      "chain_id": 43114,
      "gas_price": "25",
      "gas_limit": 25000,
      "timeout_seconds": 30,
      "retry_attempts": 3
    },
    "sui_mainnet": {
      "name": "Sui Mainnet",
      "rpc_url": "https://fullnode.mainnet.sui.io:443",
      "timeout_seconds": 30,
      "retry_attempts": 3
    }
  },
  "storage": {
    "max_data_size": 1048576,
    "chunk_size": 32768,
    "enable_compression": true,
    "enable_encryption": true
  }
}
```

### 3. 基本使用

```rust
use blockchain_store::{BlockchainManager, BlockchainConfig, BlockchainType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = BlockchainConfig::from_file("config.json")?;
    
    // 创建管理器
    let mut manager = BlockchainManager::new(config);
    manager.initialize().await?;
    
    // 存储数据
    let data = b"Hello, Blockchain!";
    let tx = manager.store_data(
        &BlockchainType::Ethereum,
        "my_key",
        data,
        None
    ).await?;
    
    println!("交易哈希: {}", tx.tx_hash);
    
    // 检索数据
    let retrieved_data = manager.retrieve_data(
        &BlockchainType::Ethereum,
        "my_key"
    ).await?;
    
    println!("检索到的数据: {:?}", retrieved_data);
    
    Ok(())
}
```

## 架构设计

### 核心组件

```
┌─────────────────────────────────────────────────────────────┐
│                区块链存储模块 (Blockchain Store)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  配置管理    │  │  管理器      │  │  错误处理    │        │
│  │ Config      │  │ Manager     │  │ Error       │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  以太坊      │  │  Solana     │  │  Cosmos     │        │
│  │ Ethereum    │  │  Storage    │  │  Storage    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              统一存储接口 (Storage Traits)               │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 接口设计

#### BlockchainStorage Trait

```rust
#[async_trait]
pub trait BlockchainStorage: Send + Sync {
    async fn store_data(&self, key: &str, data: &[u8], metadata: Option<serde_json::Value>) -> Result<StorageTransaction>;
    async fn retrieve_data(&self, key: &str) -> Result<Vec<u8>>;
    async fn verify_data(&self, key: &str, expected_hash: &str) -> Result<bool>;
    async fn get_metadata(&self, key: &str) -> Result<StorageMetadata>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn delete_data(&self, key: &str) -> Result<StorageTransaction>;
    async fn get_stats(&self) -> Result<StorageStats>;
    fn get_blockchain_type(&self) -> BlockchainType;
    fn get_network_config(&self) -> &NetworkConfig;
}
```

#### BlockchainClient Trait

```rust
#[async_trait]
pub trait BlockchainClient: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    async fn get_block_height(&self) -> Result<u64>;
    async fn get_balance(&self, address: &str) -> Result<u128>;
    async fn estimate_gas(&self, data: &[u8]) -> Result<u64>;
    async fn send_transaction(&self, data: &[u8]) -> Result<String>;
    async fn wait_for_confirmation(&self, tx_hash: &str) -> Result<StorageTransaction>;
}
```

## 配置说明

### 网络配置

每个区块链网络需要配置以下参数：

- `name`: 网络名称
- `rpc_url`: RPC 节点 URL
- `chain_id`: 链 ID（可选）
- `gas_price`: Gas 价格（可选）
- `gas_limit`: Gas 限制（可选）
- `timeout_seconds`: 超时时间
- `retry_attempts`: 重试次数

### 存储配置

- `max_data_size`: 最大数据大小（字节）
- `chunk_size`: 数据分片大小
- `enable_compression`: 是否启用压缩
- `enable_encryption`: 是否启用加密
- `backup_strategy`: 备份策略

### 重试配置

- `max_attempts`: 最大重试次数
- `initial_delay_ms`: 初始延迟（毫秒）
- `max_delay_ms`: 最大延迟（毫秒）
- `delay_multiplier`: 延迟倍数

## 使用场景

### 1. 投票数据存储

```rust
// 存储投票承诺
let commitment_data = serde_json::to_vec(&commitment)?;
let tx = manager.store_data(
    &BlockchainType::Ethereum,
    &format!("commitment_{}", vote_id),
    &commitment_data,
    Some(json!({"type": "commitment", "vote_id": vote_id}))
).await?;
```

### 2. 配置数据存储

```rust
// 存储系统配置
let config_data = serde_json::to_vec(&config)?;
let tx = manager.store_data(
    &BlockchainType::Solana,
    "system_config",
    &config_data,
    Some(json!({"type": "config", "version": "1.0"}))
).await?;
```

### 3. 事件日志存储

```rust
// 存储事件日志
let event_data = serde_json::to_vec(&event)?;
let tx = manager.store_data(
    &BlockchainType::Cosmos,
    &format!("event_{}", event_id),
    &event_data,
    Some(json!({"type": "event", "timestamp": chrono::Utc::now()}))
).await?;
```

### 4. 多链数据存储

```rust
// 存储到 Archway（开发者奖励）
let archway_tx = manager.store_data(
    &BlockchainType::Archway,
    "archway_data",
    data,
    Some(json!({"type": "archway_storage"}))
).await?;

// 存储到 Injective（去中心化交易数据）
let injective_tx = manager.store_data(
    &BlockchainType::Injective,
    "injective_data",
    data,
    Some(json!({"type": "injective_storage"}))
).await?;

// 存储到 Avalanche（高吞吐量）
let avalanche_tx = manager.store_data(
    &BlockchainType::Avalanche,
    "avalanche_data",
    data,
    Some(json!({"type": "avalanche_storage"}))
).await?;

// 存储到 Sui（对象存储）
let sui_tx = manager.store_data(
    &BlockchainType::Sui,
    "sui_data",
    data,
    Some(json!({"type": "sui_storage"}))
).await?;
```

## 错误处理

模块提供了完善的错误处理机制：

```rust
use blockchain_store::{BlockchainError, Result};

match manager.store_data(&BlockchainType::Ethereum, "key", data, None).await {
    Ok(tx) => println!("存储成功: {}", tx.tx_hash),
    Err(BlockchainError::Network(msg)) => println!("网络错误: {}", msg),
    Err(BlockchainError::InsufficientFunds(msg)) => println!("余额不足: {}", msg),
    Err(BlockchainError::TransactionFailed(msg)) => println!("交易失败: {}", msg),
    Err(e) => println!("其他错误: {}", e),
}
```

## 性能优化

### 1. 连接池

每个区块链客户端都维护连接池，提高并发性能。

### 2. 批量操作

支持批量存储和检索操作，减少网络开销。

### 3. 缓存机制

内置缓存机制，提高数据访问速度。

### 4. 异步操作

全面支持异步操作，提高系统吞吐量。

## 监控和统计

模块提供详细的统计信息：

```rust
let stats = manager.get_all_stats().await?;

for (blockchain_type, stat) in stats {
    println!("{:?} 统计:", blockchain_type);
    println!("  总交易数: {}", stat.total_transactions);
    println!("  总数据大小: {} bytes", stat.total_data_size);
    println!("  平均 Gas 使用量: {:.2}", stat.average_gas_used);
    println!("  成功率: {:.2}%", stat.success_rate * 100.0);
}
```

## 区块链特性详解

### Archway
- **开发者奖励**: 智能合约开发者可以获得网络费用奖励
- **Cosmos 生态**: 基于 Cosmos SDK，支持 IBC 跨链
- **低费用**: 相比以太坊主网费用更低
- **智能合约**: 支持 CosmWasm 智能合约

### Injective
- **去中心化交易**: 专注于去中心化衍生品交易
- **跨链支持**: 支持多链资产交易
- **高性能**: 优化的交易处理性能
- **模块化设计**: 基于 Cosmos SDK 的模块化架构

### Avalanche
- **高吞吐量**: 支持高并发交易处理
- **以太坊兼容**: 完全兼容以太坊工具和智能合约
- **子网架构**: 支持自定义子网和验证器
- **快速确认**: 亚秒级交易确认时间

### Sui
- **对象存储**: 基于对象的存储模型
- **并行执行**: 支持并行交易处理
- **Move 语言**: 使用 Move 智能合约语言
- **高性能**: 优化的执行引擎

## 扩展开发

### 添加新的区块链支持

1. 实现 `BlockchainStorage` 和 `BlockchainClient` traits
2. 在 `BlockchainManager` 中注册新的区块链类型
3. 更新配置结构以支持新的区块链参数

### 示例：添加 Polygon 支持

```rust
// 在 src/polygon.rs 中实现
pub struct PolygonStorage {
    // 实现细节
}

#[async_trait]
impl BlockchainStorage for PolygonStorage {
    // 实现所有必需的方法
}
```

## 安全考虑

1. **私钥管理**: 私钥应该加密存储，不要硬编码在配置文件中
2. **网络安全**: 使用 HTTPS 连接 RPC 节点
3. **数据验证**: 始终验证从区块链检索的数据完整性
4. **访问控制**: 实现适当的访问控制机制
5. **审计日志**: 记录所有区块链操作

## 故障排除

### 常见问题

1. **连接失败**: 检查 RPC URL 和网络连接
2. **交易失败**: 检查 Gas 设置和账户余额
3. **数据检索失败**: 确认交易已确认且数据存在
4. **配置错误**: 验证配置文件格式和参数

### 调试模式

启用调试日志：

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 许可证

MIT License

## 运行示例

### 基本使用示例
```bash
cargo run --example usage
```

### 集成存储示例
```bash
cargo run --example integration
```

### 新区块链功能展示
```bash
cargo run --example new_blockchains
```

### 运行测试
```bash
cargo test
```

### 运行集成测试
```bash
cargo test --test integration_tests
```

## 联系方式

- 项目主页: https://github.com/luckee-dao/decentralized_decision_vote
- 问题反馈: https://github.com/luckee-dao/decentralized_decision_vote/issues
