# decentralized_decision_vote

基于比特承诺模型的去中心化随机数生成 CLI（Commit–Reveal）。

## 安装

```bash
# 进入项目目录并可编辑安装
pip install -e .
# 或使用 pipx（推荐全局隔离）
pipx install .
```

需要 Python 3.9+。

## 快速开始

```bash
# 1) 初始化随机数生成会话（ID 为 session1）
decentralized-decision-vote init --id session1 --title "抽奖活动：从10人中选出1个中奖者" 

# 2) 生成承诺（本地保存盐值），仅提交承诺
decentralized-decision-vote commit --id session1 --participant alice --randomness-file alice.salt

# 3) 公布（揭示）随机数：使用相同盐值
decentralized-decision-vote reveal --id session1 --participant alice --randomness-file alice.salt

# 4) 计算中奖者
decentralized-decision-vote select --id session1

# 5) 验证（对比揭示与承诺）
decentralized-decision-vote verify --id session1

# 6) 查看状态
decentralized-decision-vote status --id session1
```

## 数据位置

默认在当前工作目录下创建 `.decentralized_decision_vote/<session_id>/`，包含：
- `meta.json`：会话元数据
- `commitments.json`：`participant -> commitment`
- `reveals.json`：`participant -> {randomness, salt}`

## 原理（简述）
- 使用 `SHA-256("commit|" + randomness + "|" + salt)` 作为承诺。
- 先提交承诺，后在揭示阶段提交 `randomness + salt`，任何人可校验。
- 所有参与者的随机数通过XOR运算生成最终随机种子，用于公平选择中奖者。

## 免责声明
该实现用于原型与教学，未包含抗女巫、匿名性增强、抗重放与链上集成等高级特性。
