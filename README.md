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
- `sqlite3` CLI

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

启用文件型本地持久化模式：

```bash
FA_DATA_DIR=/tmp/fa-data cargo run -p fa-server
```

启用 SQLite 本地数据库模式：

```bash
FA_SQLITE_DB_PATH=/tmp/fa-dev/fa.db cargo run -p fa-server
```

默认监听：

```text
FA_SERVER_ADDR=0.0.0.0:8000
```

本地端口约定：

- `FA` 默认占用 `8000`
- 如本机已有其他项目使用 `8000`，通过 `FA_SERVER_ADDR` 覆盖，不要直接改代码默认值
- 端口分配与冲突规避说明见 [docs/development/local-environment.md](/Users/admin/Documents/WORK/ai/fa/docs/development/local-environment.md)
- 如需在本地保留任务与审计历史，可通过 `FA_DATA_DIR` 启用文件模式，或通过 `FA_SQLITE_DB_PATH` 启用 SQLite 模式

运行时存储选择顺序：

- `FA_SQLITE_DB_PATH` -> SQLite 持久化模式
- `FA_DATA_DIR` -> 文件型持久化模式
- 未设置时 -> 内存模式

说明：

- SQLite 模式当前通过本机 `sqlite3` CLI 建立结构化本地持久化基线
- SQLite 基线用于本地试运行、演示和耐久性验证，不等同于最终企业级数据库方案
- 为避免与其他本地项目互相污染，`FA_SQLITE_DB_PATH` 应使用独立路径，不要与其他服务共用数据库文件

## API 起步接口

健康检查：

```bash
curl -sS http://127.0.0.1:8000/healthz
```

查看平台蓝图：

```bash
curl -sS http://127.0.0.1:8000/api/v1/blueprint | jq
```

查看审计事件：

```bash
curl -sS http://127.0.0.1:8000/api/v1/audit/events | jq
```

按 `correlation_id` 过滤审计事件：

```bash
curl -sS 'http://127.0.0.1:8000/api/v1/audit/events?correlation_id=demo-approve-001' | jq
```

回放单个任务的审计事件：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/audit-events | jq
```

查看单个任务的结构化 evidence 快照：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/evidence | jq
```

提交任务规划请求：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/plan \
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

提交任务 intake 请求并生成可追踪任务记录：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/intake \
  -H "x-correlation-id: demo-intake-001" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0",
    "title": "Investigate spindle temperature drift",
    "description": "Diagnose repeated spindle temperature drift before the next shift.",
    "priority": "critical",
    "risk": "high",
    "initiator": {
      "id": "worker_1001",
      "display_name": "Liu Supervisor",
      "role": "Production Supervisor"
    },
    "stakeholders": [],
    "equipment_ids": ["eq_cnc_01"],
    "integrations": ["mes", "cmms"],
    "desired_outcome": "Recover stable spindle temperature within tolerance",
    "requires_human_approval": true,
    "requires_diagnostic_loop": true
  }' | jq
```

说明：

- `tasks/intake` 现在会返回 `correlation_id`、`planned_task`、`context_reads` 和 `evidence`
- `context_reads` 由当前内置的 mock `MES` / mock `CMMS` connector 生成
- `evidence` 是从 connector 读取结果提炼出的结构化任务证据快照
- 审计事件可通过 `/api/v1/audit/events` 查看，也支持 `task_id / correlation_id / kind / approval_id` 过滤
- 单任务 evidence 可通过 `/api/v1/tasks/{task_id}/evidence` 查看
- 单任务审计回放可通过 `/api/v1/tasks/{task_id}/audit-events` 查看
- `priority` 当前只接受 `routine / expedited / critical`
- `risk` 当前只接受 `low / medium / high / critical`

查询已跟踪任务：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0 | jq
```

批准待审批任务：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/approve \
  -H "x-correlation-id: demo-approve-001" \
  -H "Content-Type: application/json" \
  -d '{
    "decided_by": {
      "id": "worker_2001",
      "display_name": "Chen QE",
      "role": "Quality Engineer"
    },
    "approved": true,
    "comment": "Proceed to execution"
  }' | jq
```

驳回后重新发起审批：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/resubmit \
  -H "x-correlation-id: demo-resubmit-001" \
  -H "Content-Type: application/json" \
  -d '{
    "requested_by": {
      "id": "worker_1001",
      "display_name": "Liu Supervisor",
      "role": "Production Supervisor"
    },
    "comment": "Added vibration report and revised action plan"
  }' | jq
```

启动执行 stub：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/execute \
  -H "x-correlation-id: demo-execute-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_3001",
      "display_name": "Wu Maint",
      "role": "Maintenance Technician"
    },
    "note": "Execution stub started"
  }' | jq
```

完成任务：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/complete \
  -H "x-correlation-id: demo-complete-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_3001",
      "display_name": "Wu Maint",
      "role": "Maintenance Technician"
    },
    "note": "Execution finished"
  }' | jq
```

标记任务失败：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/fail \
  -H "x-correlation-id: demo-fail-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_3001",
      "display_name": "Wu Maint",
      "role": "Maintenance Technician"
    },
    "reason": "Cooling loop inspection failed"
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
- [docs/progress/README.md](/Users/admin/Documents/WORK/ai/fa/docs/progress/README.md)
- [docs/journal/README.md](/Users/admin/Documents/WORK/ai/fa/docs/journal/README.md)

## 当前状态

已完成：

- Rust workspace 初始化
- 领域模型与任务规划核心
- HTTP API 启动骨架
- 任务生命周期主链：`intake -> get -> approve -> execute -> complete / fail`
- 修订闭环：`approve(false) -> resubmit -> approve(true)`
- mock `MES` / mock `CMMS` connector 上下文读取
- 可替换 `task repository` 抽象与内存实现
- `FA_DATA_DIR` 驱动的本地文件持久化 task / audit storage
- 按任务和链路主键查询的审计回放能力
- `FA_SQLITE_DB_PATH` 驱动的 SQLite task / audit 持久化基线
- 首条 pilot workflow 候选比较与规格定义基线
- 结构化 task evidence snapshot 与任务级 evidence 查询接口
- 内存审计事件流与 `correlation_id` 贯通
- 服务层生命周期集成测试
- 基础测试
- CI / Release 基线
- 架构与项目文档基线

下一步优先级：

1. 完成 `v0.2.0` 的测试清单、发布清单与试运行验证记录
2. 为 pilot workflow 增加更明确的角色责任矩阵与审批策略表达
3. 评估 SQLite 向更强数据库后端的迁移路径
4. 扩展 evidence、审批 SLA 和异常路径表达

## 团队工作流

项目团队的标准工作流程已经固化在仓库中，后续需求、设计、开发、测试、发布和试运行都按这些文档执行：

- [docs/governance/team-operating-model.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/team-operating-model.md)
- [docs/governance/delivery-lifecycle.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/delivery-lifecycle.md)
- [docs/governance/governance-controls.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/governance-controls.md)

## 当前执行计划

当前已经把 `M1` 的执行计划细化到工作包、责任、依赖和验收层，后续实施以此为准：

- [docs/planning/m1-execution-plan.md](/Users/admin/Documents/WORK/ai/fa/docs/planning/m1-execution-plan.md)
