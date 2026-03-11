# ADR 0001: FA 平台基础架构与 Agentic 模式基线

- Status: Accepted
- Date: 2026-03-11

## Context

本项目从 0-1 面向制造企业构建 Agentic AI 协同平台。制造场景有几个天然约束：

- 任务通常跨人、系统、设备，不是单轮问答
- 风险和责任边界明确，不能依赖黑盒自动决策
- 现场设备与企业系统写操作必须遵守 SOP 和审批规则
- 平台需要长期演进，因此需要强类型、可维护和可部署的技术基线

同时，Google Cloud 的 Agentic AI design pattern 选择框架提示：不同业务问题应匹配不同的 agentic pattern，而不是默认使用复杂多 agent。

## Decision

我们做出以下决策：

1. 核心语言使用 Rust。
2. 仓库采用 workspace 结构，拆分 `fa-domain`、`fa-core`、`fa-server`。
3. 第一阶段主模式采用：
   - `coordinator`
   - `ReAct loop`
   - `human-in-the-loop`
   - `deterministic workflow`
   - `custom business logic`
4. `single-agent` 仅用于低风险、窄任务。
5. 所有高风险任务默认进入审批策略。
6. 所有外部写操作和设备相关动作必须先由确定性业务逻辑门控。

## Consequences

正面影响：

- 架构与制造治理场景匹配
- 能较早进入真实业务流程，而非停留在 demo
- Rust 有利于长期维护、并发、接口清晰和部署稳定性

代价：

- 起步阶段实现速度可能慢于脚本式原型
- 需要更严格的模型与边界设计
- 集成层抽象要做得更扎实

## Rejected alternatives

### 1. 直接做通用聊天型 Copilot

拒绝原因：无法进入企业真实流程，也无法承担审批和执行闭环。

### 2. 一开始就做完全自治多 Agent swarm

拒绝原因：制造场景不适合先上高复杂度自治模型，治理风险过高。

### 3. 使用弱类型快速脚本堆叠全部核心能力

拒绝原因：短期快，长期会在设备接入、审计和版本治理阶段积累不可控技术债。
