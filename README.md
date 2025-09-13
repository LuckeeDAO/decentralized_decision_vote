# decentralized_decision_vote

基于比特承诺模型的去中心化随机数生成系统（Commit–Reveal）。

## 项目架构

```
decentralized_decision_vote/
├── service/           # 单体服务端应用
│   ├── src/          # 服务端源代码
│   ├── Cargo.toml    # 服务端依赖配置
│   └── Dockerfile    # 服务端容器配置
├── services/          # 微服务集合
│   ├── vote-api/     # 投票 API 服务
│   ├── admin-api/    # 管理 API 服务
│   └── notification-service/  # 通知服务
├── clients/           # 客户端工具和 SDK
│   ├── cli/          # 命令行工具
│   └── sdk/          # 多语言 SDK
│       ├── javascript/  # JavaScript/TypeScript SDK
│       ├── python/      # Python SDK
│       └── rust/        # Rust SDK
├── core/              # 核心业务逻辑
├── shared/            # 共享代码库
├── storage/           # 数据存储层
└── infrastructure/    # 基础设施配置
```

## 安装

### 服务端安装

```bash
# 进入服务端目录
cd service

# 构建服务端
cargo build --release

# 运行服务端
cargo run
```

### 客户端安装

```bash
# 安装 CLI 工具
cd clients/cli
cargo install --path .

# 安装 Python SDK
cd clients/sdk/python
pip install -e .

# 安装 JavaScript SDK
cd clients/sdk/javascript
npm install
```

需要 Rust 1.70+ 和 Python 3.9+。

## 快速开始

### 1. 启动服务端

```bash
# 启动单体服务端
cd service
cargo run

# 或启动微服务架构
cd infrastructure/docker
docker-compose up -d
```

### 2. 使用 CLI 工具

```bash
# 安装 CLI 工具
cd clients/cli
cargo install --path .

# 创建投票
vote create --title "抽奖活动：从10人中选出1个中奖者" --description "公平抽奖" --template "lottery" --commitment-hours 24 --reveal-hours 24

# 提交承诺
vote commit --vote-id <vote_id> --voter alice --value "option1" --salt <salt>

# 揭示投票
vote reveal --vote-id <vote_id> --voter alice --value "option1" --salt <salt>

# 查看结果
vote results --vote-id <vote_id>
```

### 3. 使用 SDK

```python
# Python SDK 示例
from luckee_dao_vote_sdk import VoteClient

client = VoteClient({"base_url": "http://localhost:8080"})
vote_id = await client.create_vote({
    "title": "抽奖活动",
    "template_id": "lottery",
    "commitment_duration_hours": 24,
    "reveal_duration_hours": 24
})
```

```typescript
// JavaScript SDK 示例
import { VoteClient } from '@luckee-dao/vote-sdk';

const client = new VoteClient({ baseUrl: 'http://localhost:8080' });
const voteId = await client.createVote({
  title: '抽奖活动',
  template_id: 'lottery',
  commitment_duration_hours: 24,
  reveal_duration_hours: 24
});
```

## 数据存储

### 服务端数据
- **单体服务端**：数据存储在 `service/data/` 目录
- **微服务架构**：数据存储在配置的数据库中（PostgreSQL/Redis）

### 客户端数据
- **CLI 工具**：本地数据存储在 `~/.decentralized_decision_vote/` 目录
- **SDK**：数据通过 API 与服务端交互，不存储本地数据

### 数据格式
```
vote_data/
├── <vote_id>/
│   ├── meta.json          # 投票元数据
│   ├── commitments.json   # 承诺数据
│   ├── reveals.json       # 揭示数据
│   └── results.json       # 结果数据
```

## 架构选择

### 单体服务端 (`service/`)
- **适用场景**：小型项目、快速原型、单机部署
- **优势**：简单部署、易于调试、资源占用少
- **劣势**：扩展性有限、单点故障

### 微服务架构 (`services/`)
- **适用场景**：大型项目、高可用性要求、云原生部署
- **优势**：高可扩展性、服务独立、容错性强
- **劣势**：复杂度高、运维成本高

### 客户端工具 (`clients/`)
- **CLI 工具**：适合脚本化、自动化场景
- **SDK**：适合集成到现有应用中

## 原理（简述）
- 使用 `SHA-256("commit|" + randomness + "|" + salt)` 作为承诺。
- 先提交承诺，后在揭示阶段提交 `randomness + salt`，任何人可校验。
- 所有参与者的随机数通过XOR运算生成最终随机种子，用于公平选择中奖者。

## 免责声明
该实现用于原型与教学，未包含抗女巫、匿名性增强、抗重放与链上集成等高级特性。
