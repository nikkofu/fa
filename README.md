# FA

FA 是一个面向生产制造型企业的 Agentic AI 协同平台，目标是让 AI、Agent、企业系统、人员与设备在统一治理框架下协同工作。

当前仓库为 `v0.1.0` 起步版本，采用 Rust 作为核心实现语言，先建立可扩展的领域模型、任务编排核心与 HTTP 服务底座，再逐步接入企业业务系统与工厂边缘场景。

## 为什么现在做

制造企业已经拥有 ERP、MES、CMMS、SCADA、QMS、仓储系统与现场设备，但这些系统之间仍存在三个核心断层：

1. 数据能流动，但决策不能协同。
2. AI 能回答问题，但不能在可治理前提下参与工作流。
3. 现场设备、班组长、工程师与企业系统缺少统一的执行和审批闭环。

FA 的定位不是一个“聊天机器人”，而是一个可审计、可扩展、可接入企业 SOP 的制造协同操作层。

## 第一阶段设计原则

本仓库的第一阶段架构，参考了 Google Cloud 关于 Agentic AI system design pattern 的模式选择方法：

- 低风险、窄任务优先 `single-agent`
- 跨系统、跨角色任务优先 `coordinator`
- 诊断与异常处理优先 `ReAct loop`
- 安全、质量、成本关键动作必须 `human-in-the-loop`
- 涉及设备动作或业务写操作时必须包裹 `deterministic workflow + custom business logic`

这里的关键不是追求“最多的 agent”，而是优先保证工厂语境下的安全性、可追溯性和业务可解释性。

## 当前仓库结构

```text
.
├── apps/fa-server          # HTTP API 入口
├── crates/fa-core          # 编排核心、蓝图与任务规划逻辑
├── crates/fa-domain        # 企业、人员、设备、任务等领域模型
├── docs                    # 架构、ADR、路线图、项目管理与发布文档
├── .github/workflows       # CI 与 Release 自动化
├── CHANGELOG.md            # 版本变更记录
└── Makefile                # 常用开发命令
```

## 快速开始

要求：

- Rust `1.84.1`
- Cargo `1.84.1`

开发命令：

```bash
make fmt
make lint
make test
make run
```

或直接运行：

```bash
cargo run -p fa-server
```

默认监听：

```text
FA_SERVER_ADDR=0.0.0.0:8080
```

## API 起步接口

健康检查：

```bash
curl -sS http://127.0.0.1:8080/healthz
```

查看平台蓝图：

```bash
curl -sS http://127.0.0.1:8080/api/v1/blueprint | jq
```

提交任务规划请求：

```bash
curl -sS http://127.0.0.1:8080/api/v1/tasks/plan \
  -H "Content-Type: application/json" \
  -d '{
    "id": "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0",
    "title": "Investigate spindle temperature drift",
    "description": "Diagnose repeated spindle temperature drift before the next shift.",
    "priority": "expedited",
    "risk": "medium",
    "initiator": {
      "id": "worker_1001",
      "display_name": "Liu Supervisor",
      "role": "Production Supervisor"
    },
    "stakeholders": [],
    "equipment_ids": ["eq_cnc_01"],
    "integrations": ["mes", "cmms"],
    "desired_outcome": "Recover stable spindle temperature within tolerance",
    "requires_human_approval": false,
    "requires_diagnostic_loop": true
  }' | jq
```

## 仓库治理

- Git 远端已配置为 `https://github.com/nikkofu/fa.git`
- 采用 `semver`
- `CHANGELOG.md` 记录版本说明
- GitHub Actions 负责 `fmt / clippy / test / release`
- Release 通过 `v*` tag 触发

详细说明见：

- [docs/architecture.md](/Users/admin/Documents/WORK/ai/fa/docs/architecture.md)
- [docs/project-charter.md](/Users/admin/Documents/WORK/ai/fa/docs/project-charter.md)
- [docs/roadmap.md](/Users/admin/Documents/WORK/ai/fa/docs/roadmap.md)
- [docs/release-process.md](/Users/admin/Documents/WORK/ai/fa/docs/release-process.md)
- [docs/governance/README.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/README.md)
- [docs/planning/README.md](/Users/admin/Documents/WORK/ai/fa/docs/planning/README.md)

## 当前状态

已完成：

- Rust workspace 初始化
- 领域模型与任务规划核心
- HTTP API 启动骨架
- 基础测试
- CI / Release 基线
- 架构与项目文档基线

下一步优先级：

1. 引入持久化与审计存储
2. 定义 connector trait，接入 ERP / MES / CMMS 只读链路
3. 建立审批流、执行流与回滚策略
4. 建立试运行场景与验收标准

## 团队工作流

项目团队的标准工作流程已经固化在仓库中，后续需求、设计、开发、测试、发布和试运行都按这些文档执行：

- [docs/governance/team-operating-model.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/team-operating-model.md)
- [docs/governance/delivery-lifecycle.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/delivery-lifecycle.md)
- [docs/governance/governance-controls.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/governance-controls.md)

## 当前执行计划

当前已经把 `M1` 的执行计划细化到工作包、责任、依赖和验收层，后续实施以此为准：

- [docs/planning/m1-execution-plan.md](/Users/admin/Documents/WORK/ai/fa/docs/planning/m1-execution-plan.md)
