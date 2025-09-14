# Decentralized Decision Vote - 去中心化决策投票工具

## 项目概述

Decentralized Decision Vote 是一个基于区块链的去中心化决策投票工具，专门用于实现去中心化随机数生成。该工具采用承诺-揭示(Commit-Reveal)机制，确保随机数生成的公平性、透明性和可验证性，可广泛应用于企业年会抽奖、盲盒等级设置、扑克自动洗牌等业务场景。

## 核心特性

- **去中心化随机数生成**: 基于承诺-揭示机制的安全随机数生成
- **五条条件保证**: 满足去中心化决策的五个核心条件
- **多业务场景支持**: 企业抽奖、盲盒设置、扑克牌生成等
- **区块链存储**: 支持多种区块链网络的数据存储
- **公平透明**: 完全公开可验证的随机数生成过程
- **高性能**: 基于 Rust + Axum 的高性能服务
- **微服务架构**: 模块化的微服务设计
- **CLI 工具**: 功能完整的命令行界面

## 业务场景

### 企业年会抽奖
- **场景描述**: 企业年会抽奖需要公平、透明的随机数生成
- **解决方案**: 使用承诺-揭示机制，确保抽奖过程无法被操控
- **特点**: 完全公开可验证，员工可以验证抽奖结果的公平性

### 盲盒等级设置
- **场景描述**: 盲盒游戏中需要随机分配不同等级的奖品
- **解决方案**: 基于区块链的随机数生成，确保等级分配的随机性
- **特点**: 防止开发者操控概率，保证玩家权益

### 扑克牌生成
- **场景描述**: 在线扑克游戏中需要生成公平的牌组
- **解决方案**: 使用去中心化随机数生成牌组，确保游戏公平
- **特点**: 无法预测牌组顺序，保证游戏公正性

### 其他应用场景
- **彩票系统**: 公平的号码生成
- **游戏道具**: 随机属性分配
- **抽签系统**: 公平的抽签结果
- **随机分组**: 公平的团队分组

## 五条条件保证

本工具满足去中心化决策的五个核心条件：

1. **平等性 (Equality)**: 采用同质化代币作为权益度量单位
2. **透明与可验证 (Transparency & Verifiability)**: 投票与决策过程公开且可独立复核
3. **自由性与结果承诺性 (Freedom & Outcome Commitment)**: 任何合规参与者可自由表达偏好，系统对结果具有承诺性，确保决策结果的可信度
4. **一致性 (Strict Unanimity)**: 严格"全体投票人一致同意"原则
5. **抗合谋与反策略性 (Collusion Resistance)**: 机制层面抑制联合操控，防止参与者通过伪造偏好获得策略性稳定收益

## 架构设计

### 整体架构

```
┌──────────────────────────────────────────────────────────────────┐
│                Decentralized Decision Vote                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐  │
│  │  API Services   │    │  Core Services  │    │  Storage     │  │
│  │                 │    │                 │    │  Services    │  │
│  │ • Vote API      │    │ • Vote Engine   │    │ • Vote Store │  │
│  │ • Admin API     │    │ • Template      │    │ • Event Store│  │
│  │ • Notification  │    │ • Commitment    │    │ • Config     │  │
│  │   Service       │    │   Engine        │    │   Store      │  │
│  │                 │    │                 │    │ • Blockchain │  │
│  │                 │    │                 │    │   Store      │  │
│  └─────────────────┘    └─────────────────┘    └──────────────┘  │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    共享服务层                                │ │
│  │                                                             │ │
│  │ ┌─────────────┐ ┌─────────────┐ ┌──────────────┐            │ │
│  │ │ Shared      │ │ Shared      │ │ Shared       │            │ │
│  │ │ Types       │ │ Config      │ │ Utils        │            │ │
│  │ └─────────────┘ └─────────────┘ └──────────────┘            │ │
│  │                                                             │ │
│  │ ┌─────────────┐ ┌─────────────┐ ┌───────────────┐           │ │
│  │ │ Shared      │ │ Clients     │ │ Infrastructure│           │ │
│  │ │ Logging     │ │ CLI/SDK     │ │               │           │ │
│  │ └─────────────┘ └─────────────┘ └───────────────┘           │ │
│  └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### 核心组件

#### 1. API Services (API服务)

**Vote API** (`services/vote-api/`)
- 投票相关的 REST API
- 投票会话管理
- 投票数据查询
- 投票结果计算

**Admin API** (`services/admin-api/`)
- 管理相关的 REST API
- 系统配置管理
- 用户管理
- 统计监控

**Notification Service** (`services/notification-service/`)
- 通知服务
- 实时消息推送
- 邮件通知
- 短信通知

#### 2. Core Services (核心服务)

**Vote Engine** (`core/vote-engine/`)
- 随机数生成引擎核心
- 承诺-揭示机制实现
- 随机数算法处理
- 结果计算和验证

**Template System** (`core/template-system/`)
- 随机数生成模板系统
- 业务场景模板管理
- 模板渲染和配置
- 模板验证和校验

**Commitment Engine** (`core/commitment-engine/`)
- 承诺引擎核心
- 比特承诺协议实现
- 承诺哈希验证
- 揭示阶段处理

#### 3. Storage Services (存储服务)

**Vote Store** (`storage/vote-store/`)
- 随机数生成数据存储
- 承诺和揭示数据存储
- 生成历史记录存储
- 统计和分析数据存储

**Event Store** (`storage/event-store/`)
- 事件存储
- 事件溯源
- 事件重放
- 事件查询

**Config Store** (`storage/config-store/`)
- 配置存储
- 系统配置
- 用户配置
- 配置版本管理

**Blockchain Store** (`storage/blockchain-store/`)
- 区块链存储
- 多链支持（Ethereum、Solana、Cosmos、Archway、Injective、Avalanche、Sui）
- 统一存储接口
- 配置驱动的区块链访问

#### 4. Shared Services (共享服务)

**Shared Types** (`shared/types/`)
- 共享数据类型
- 数据结构定义
- 类型转换
- 序列化支持

**Shared Config** (`shared/config/`)
- 共享配置管理
- 环境配置
- 配置验证
- 配置热更新

**Shared Utils** (`shared/utils/`)
- 共享工具函数
- 通用算法
- 加密工具
- 验证工具

**Shared Logging** (`shared/logging/`)
- 共享日志系统
- 日志格式化
- 日志级别管理
- 日志聚合

#### 5. Client Tools (客户端工具)

**CLI Tool** (`clients/cli/`)
- 命令行界面
- 投票管理
- 模板管理
- 结果查询
- 系统监控

**SDK** (`clients/sdk/`)
- 多语言 SDK
- API 封装
- 类型定义
- 工具函数

## 技术特性

### 1. 微服务架构

**服务拆分**:
- 按功能域拆分服务
- 独立的数据库
- 独立的部署
- 独立的扩展

**服务通信**:
- HTTP REST API
- WebSocket 实时通信
- 消息队列
- 事件总线

**服务发现**:
- 服务注册
- 服务发现
- 负载均衡
- 健康检查

### 2. 高性能设计

**异步处理**:
- 基于 Tokio 的异步运行时
- 非阻塞 I/O
- 并发处理
- 资源池管理

**缓存策略**:
- 多级缓存
- 缓存预热
- 缓存失效
- 缓存统计

**数据库优化**:
- 连接池
- 查询优化
- 索引优化
- 分片策略

**区块链存储**:
- 多链支持
- 统一接口
- 配置驱动
- 异步操作

### 3. 事件驱动架构

**事件类型**:
- **系统事件**: 系统级事件
- **业务事件**: 业务逻辑事件
- **用户事件**: 用户操作事件
- **错误事件**: 错误和异常事件

**事件处理**:
- 事件发布
- 事件订阅
- 事件路由
- 事件持久化

### 4. 可扩展性

**水平扩展**:
- 无状态服务
- 负载均衡
- 数据分片
- 缓存集群

**垂直扩展**:
- 资源优化
- 性能调优
- 监控告警
- 自动扩缩容

## 项目结构

```
decentralized_decision_vote/
├── services/                    # 微服务
│   ├── vote-api/               # 投票API服务
│   │   ├── src/main.rs         # 服务主程序
│   │   └── Cargo.toml          # 服务依赖
│   ├── admin-api/              # 管理API服务
│   └── notification-service/   # 通知服务
├── core/                       # 核心服务
│   ├── vote-engine/            # 投票引擎
│   ├── template-system/        # 模板系统
│   └── commitment-engine/      # 承诺引擎
├── storage/                    # 存储服务
│   ├── vote-store/             # 投票存储
│   ├── event-store/            # 事件存储
│   ├── config-store/           # 配置存储
│   └── blockchain-store/       # 区块链存储
├── shared/                     # 共享服务
│   ├── types/                  # 共享类型
│   ├── config/                 # 共享配置
│   ├── utils/                  # 共享工具
│   └── logging/                # 共享日志
├── clients/                    # 客户端
│   ├── cli/                    # 命令行工具
│   │   ├── src/main.rs         # CLI 主程序
│   │   ├── src/commands.rs     # 命令实现
│   │   ├── src/client.rs       # API 客户端
│   │   └── Cargo.toml          # CLI 依赖
│   └── sdk/                    # SDK
├── infrastructure/             # 基础设施
│   ├── docker/                 # Docker配置
│   ├── kubernetes/             # K8s配置
│   └── monitoring/             # 监控配置
├── tests/                      # 测试
│   ├── unit/                   # 单元测试
│   ├── integration/            # 集成测试
│   └── e2e/                    # 端到端测试
├── Cargo.toml                  # 工作空间配置
└── README.md                   # 项目说明
```

## 快速开始

### 前置条件

- Rust 1.70+
- PostgreSQL 12+ (可选)
- Redis 6+ (可选)
- Docker (可选)

### 安装依赖

```bash
# 克隆项目
git clone https://github.com/luckee-dao/decentralized_decision_vote.git
cd decentralized_decision_vote

# 安装Rust依赖
cargo build
```

### 配置环境

```bash
# 设置环境变量
export DATABASE_URL="postgresql://user:password@localhost:5432/luckee_vote"
export REDIS_URL="redis://localhost:6379"
export RUST_LOG="info"

# 区块链存储配置（可选）
export BLOCKCHAIN_CONFIG_PATH="storage/blockchain-store/examples/config.json"
```

### 运行服务

```bash
# 运行投票API服务
cargo run --bin vote-api

# 运行管理API服务
cargo run --bin admin-api

# 运行通知服务
cargo run --bin notification-service

# 运行 CLI 工具
cargo run --bin vote --help
```

### Docker 部署

```bash
# 构建镜像
docker build -t luckee-vote-api .

# 运行容器
docker run -p 8080:8080 luckee-vote-api
```

## API 接口

### 随机数生成API

**创建随机数生成会话**:
```http
POST /api/v1/sessions
Content-Type: application/json

{
  "session_id": "lottery_2024_001",
  "scenario_type": "enterprise_lottery",
  "participants": ["employee1", "employee2", "employee3"],
  "prize_count": 10,
  "commitment_duration_hours": 24,
  "reveal_duration_hours": 24
}
```

**提交随机数承诺**:
```http
POST /api/v1/sessions/{session_id}/commitments
Content-Type: application/json

{
  "participant_id": "employee1",
  "random_seed": "encrypted_random_seed",
  "commitment_hash": "sha256_hash_of_seed"
}
```

**揭示随机数**:
```http
POST /api/v1/sessions/{session_id}/reveals
Content-Type: application/json

{
  "participant_id": "employee1",
  "random_seed": "original_random_seed",
  "salt": "random_salt_value"
}
```

**获取随机数结果**:
```http
GET /api/v1/sessions/{session_id}/results
```

### 管理API

**获取系统状态**:
```http
GET /api/v1/admin/status
```

**获取统计信息**:
```http
GET /api/v1/admin/stats
```

**更新系统配置**:
```http
PUT /api/v1/admin/config
Content-Type: application/json

{
  "voting_timeout": 3600,
  "max_participants": 1000
}
```

### WebSocket 接口

**实时数据推送**:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};
```

## CLI 命令行工具

### 基本使用

**查看帮助信息**:
```bash
cargo run --bin vote --help
```

**设置 API 服务器地址**:
```bash
cargo run --bin vote --api-url http://localhost:8080 <command>
```

### 随机数生成管理

**创建抽奖会话**:
```bash
cargo run --bin vote create \
  --title "企业年会抽奖" \
  --description "2024年企业年会抽奖活动" \
  --template "lottery" \
  --params '{"prize_count": 10, "participants": ["员工1", "员工2", "员工3"]}' \
  --commitment-hours 24 \
  --reveal-hours 24
```

**创建盲盒等级设置**:
```bash
cargo run --bin vote create \
  --title "盲盒等级设置" \
  --description "游戏道具等级随机分配" \
  --template "blind_box" \
  --params '{"levels": ["普通", "稀有", "史诗", "传说"], "probabilities": [0.6, 0.25, 0.12, 0.03]}' \
  --commitment-hours 12 \
  --reveal-hours 12
```

**创建扑克牌生成**:
```bash
cargo run --bin vote create \
  --title "扑克牌生成" \
  --description "在线扑克游戏牌组生成" \
  --template "poker_deck" \
  --params '{"deck_count": 1, "shuffle_rounds": 7}' \
  --commitment-hours 6 \
  --reveal-hours 6
```

**获取会话信息**:
```bash
cargo run --bin vote get <session_id>
```

**列出会话**:
```bash
# 列出所有会话
cargo run --bin vote list

# 按场景类型筛选
cargo run --bin vote list --status "lottery"

# 按创建者筛选
cargo run --bin vote list --creator "admin"
```

### 随机数生成操作

**提交随机数承诺**:
```bash
cargo run --bin vote commit \
  <session_id> \
  --voter "employee1" \
  --value "random_seed_12345" \
  --salt "salt_67890"
```

**揭示随机数**:
```bash
cargo run --bin vote reveal \
  <session_id> \
  --voter "employee1" \
  --value "random_seed_12345" \
  --salt "salt_67890"
```

**获取随机数结果**:
```bash
cargo run --bin vote results <session_id>
```

**验证随机数结果**:
```bash
cargo run --bin vote verify <session_id>
```

### 业务场景模板管理

**列出可用模板**:
```bash
cargo run --bin vote templates
```

**获取模板信息**:
```bash
cargo run --bin vote template lottery
cargo run --bin vote template blind_box
cargo run --bin vote template poker_deck
```

### 系统监控

**健康检查**:
```bash
cargo run --bin vote health
```

### CLI 配置

**配置文件位置**:
- Linux/macOS: `~/.config/vote-cli/config.json`
- Windows: `%APPDATA%\vote-cli\config.json`

**配置文件示例**:
```json
{
  "api_url": "http://localhost:8080",
  "default_commitment_hours": 24,
  "default_reveal_hours": 24,
  "output_format": "json"
}
```

### 高级用法

**批量操作**:
```bash
# 创建多个抽奖会话
for i in {1..5}; do
  cargo run --bin vote create \
    --title "抽奖活动 $i" \
    --description "第 $i 轮抽奖活动" \
    --template "lottery" \
    --params '{"prize_count": 5, "participants": ["员工1", "员工2", "员工3"]}'
done
```

**脚本化使用**:
```bash
#!/bin/bash
# 创建企业年会抽奖并获取结果
SESSION_ID=$(cargo run --bin vote create \
  --title "企业年会抽奖" \
  --description "2024年企业年会抽奖活动" \
  --template "lottery" \
  --params '{"prize_count": 10, "participants": ["员工1", "员工2", "员工3"]}' \
  --output json | jq -r '.session_id')

echo "创建的抽奖会话 ID: $SESSION_ID"

# 等待承诺阶段结束
sleep 86400  # 24小时

# 等待揭示阶段结束
sleep 86400  # 24小时

# 获取抽奖结果
cargo run --bin vote results $SESSION_ID
```

## 配置管理

### 环境配置

**数据库配置**:
```yaml
database:
  url: "postgresql://user:password@localhost:5432/luckee_vote"
  max_connections: 20
  connection_timeout: 30
  ssl_mode: "prefer"
```

**Redis配置**:
```yaml
redis:
  url: "redis://localhost:6379"
  max_connections: 50
  connection_timeout: 5
  key_prefix: "luckee:vote:"
```

**服务配置**:
```yaml
services:
  vote_api:
    port: 8080
    host: "0.0.0.0"
    workers: 4
  admin_api:
    port: 8081
    host: "0.0.0.0"
    workers: 2
```

### 功能配置

**随机数生成配置**:
```yaml
random_generation:
  default_timeout: 3600
  max_participants: 1000
  min_participants: 2
  commitment_phase: 1800
  reveal_phase: 1800
  max_random_range: 1000000
  min_random_range: 1
  hash_algorithm: "sha256"
```

**通知配置**:
```yaml
notifications:
  email:
    enabled: true
    smtp_host: "smtp.gmail.com"
    smtp_port: 587
  websocket:
    enabled: true
    heartbeat_interval: 30
```

**区块链存储配置**:
```yaml
blockchain:
  default_blockchain: "Ethereum"
  networks:
    ethereum_mainnet:
      name: "Ethereum Mainnet"
      rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID"
      chain_id: 1
      gas_price: "20"
      gas_limit: 21000
    solana_mainnet:
      name: "Solana Mainnet"
      rpc_url: "https://api.mainnet-beta.solana.com"
    archway_mainnet:
      name: "Archway Mainnet"
      rpc_url: "https://rpc.mainnet.archway.io"
      gas_price: "0.02"
      gas_limit: 150000
    injective_mainnet:
      name: "Injective Mainnet"
      rpc_url: "https://tm.injective.network"
      gas_price: "0.03"
      gas_limit: 180000
    avalanche_mainnet:
      name: "Avalanche C-Chain"
      rpc_url: "https://api.avax.network/ext/bc/C/rpc"
      chain_id: 43114
      gas_price: "25"
      gas_limit: 25000
    sui_mainnet:
      name: "Sui Mainnet"
      rpc_url: "https://fullnode.mainnet.sui.io:443"
  storage:
    max_data_size: 1048576
    chunk_size: 32768
    enable_compression: true
    enable_encryption: true
```

## 监控和统计

### 性能监控

**指标类型**:
- **请求指标**: 请求数量、响应时间、错误率
- **资源指标**: CPU使用率、内存使用率、磁盘使用率
- **业务指标**: 随机数生成次数、参与者数量、会话数量、成功率
- **系统指标**: 服务状态、健康检查、依赖状态

**监控工具**:
- **Prometheus**: 指标收集
- **Grafana**: 指标可视化
- **Jaeger**: 分布式追踪
- **ELK Stack**: 日志分析

### 告警配置

**告警规则**:
- **服务不可用**: 服务健康检查失败
- **响应时间过长**: 响应时间超过阈值
- **错误率过高**: 错误率超过阈值
- **资源使用过高**: 资源使用率超过阈值

**告警通知**:
- **邮件通知**: 发送邮件告警
- **短信通知**: 发送短信告警
- **Slack通知**: 发送Slack消息
- **Webhook通知**: 发送Webhook请求

## 测试

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration

# 运行端到端测试
cargo test --test e2e

# 测试 CLI 工具
cargo test --package vote-cli
```

### 测试类型

**单元测试**:
- 服务逻辑测试
- 工具函数测试
- 数据结构测试
- 算法测试

**集成测试**:
- API集成测试
- 数据库集成测试
- 服务间集成测试
- 外部依赖集成测试

**端到端测试**:
- 完整用户流程测试
- 跨服务测试
- 性能测试
- 压力测试

**CLI 测试**:
- 命令行参数测试
- 命令执行测试
- API 交互测试
- 错误处理测试

## 部署

### Docker 部署

**Dockerfile**:
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/vote-api /usr/local/bin/
EXPOSE 8080
CMD ["vote-api"]
```

**Docker Compose**:
```yaml
version: '3.8'
services:
  vote-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://user:password@db:5432/luckee_vote
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
  
  db:
    image: postgres:13
    environment:
      POSTGRES_DB: luckee_vote
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
  
  redis:
    image: redis:6
    volumes:
      - redis_data:/data
```

### Kubernetes 部署

**Deployment**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vote-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vote-api
  template:
    metadata:
      labels:
        app: vote-api
    spec:
      containers:
      - name: vote-api
        image: luckee/vote-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
```

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 相关链接

- [Luckee DAO 主项目](https://github.com/luckee-dao/decentralized_decision)
- [区块链存储模块文档](storage/blockchain-store/README.md)
- [Rust 文档](https://doc.rust-lang.org/)
- [Axum 文档](https://docs.rs/axum/)
- [Tokio 文档](https://tokio.rs/)

## 支持

如有问题或建议，请：

1. 查看 [Issues](https://github.com/luckee-dao/decentralized_decision_vote/issues)
2. 创建新的 Issue
3. 联系开发团队

---

**注意**: 这是一个实验性项目，请在生产环境中谨慎使用。