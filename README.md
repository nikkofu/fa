# FA

FA 是一个面向生产制造型企业的 Agentic AI 协同平台，目标是让 AI、Agent、企业系统、人员与设备在统一治理框架下协同工作。

当前仓库已发布 `v0.2.0`，采用 Rust 作为核心实现语言，先建立可扩展的领域模型、任务编排核心与 HTTP 服务底座，再逐步接入企业业务系统与工厂边缘场景。

本次版本说明见 [CHANGELOG.md](CHANGELOG.md) 与 [docs/release/v0.2.0-release-notes.md](docs/release/v0.2.0-release-notes.md)。

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
├── scripts                 # 本地 smoke / 验证脚本
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
make smoke
make smoke-sandbox
make release-check
make release-check-sandbox
make run
```

或直接运行：

```bash
cargo run -p fa-server
```

启用文件型本地持久化模式：

```bash
FA_DATA_DIR="$(pwd)/sandbox/fa-data" cargo run -p fa-server
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
- 端口分配与冲突规避说明见 [docs/development/local-environment.md](docs/development/local-environment.md)
- 如需在本地保留任务与审计历史，可通过 `FA_DATA_DIR` 启用文件模式，或通过 `FA_SQLITE_DB_PATH` 启用 SQLite 模式

运行时存储选择顺序：

- `FA_SQLITE_DB_PATH` -> SQLite 持久化模式
- `FA_DATA_DIR` -> 文件型持久化模式
- 未设置时 -> 内存模式

说明：

- SQLite 模式当前通过本机 `sqlite3` CLI 建立结构化本地持久化基线
- SQLite 基线用于本地试运行、演示和耐久性验证，不等同于最终企业级数据库方案
- 为避免与其他本地项目互相污染，`FA_SQLITE_DB_PATH` 应使用独立路径，不要与其他服务共用数据库文件
- 项目内 `sandbox/` 目录保留给受限环境下的本地数据、smoke 工件和临时验证资产
- `make smoke-sandbox` 使用进程内 HTTP 路由验证，不依赖本地 TCP 监听，适合沙箱或受限执行环境

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

查看单个任务的治理矩阵与审批策略：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/governance | jq
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

- `tasks/intake` 现在会返回 `correlation_id`、`planned_task`、`context_reads`、`evidence`、`follow_up_items`、`follow_up_summary`、`handoff_receipt`、`handoff_receipt_summary`、`alert_cluster_drafts` 和 `alert_triage_summary`
- `context_reads` 由当前内置的 mock `MES` / mock `CMMS` connector 生成
- `evidence` 是从 connector 读取结果提炼出的结构化任务证据快照
- `shift handoff` 与 `alert triage` 请求现在都会受控生成 1 条 seeded `follow_up_item` draft，并返回非零 `follow_up_summary`
- `shift handoff` 与 `alert triage` 请求现在都可通过显式 `follow-up-items/{follow_up_id}/accept-owner` action 把 seeded `follow_up_item` 从 `draft` 推进到 `accepted`
- `GET /api/v1/follow-up-items` 现在会跨任务聚合返回 follow-up owner queue，并支持 `task_id / source_kind / status / owner_id / owner_role / overdue_only / blocked_only / escalation_required / due_before / risk / priority` 过滤
- `GET /api/v1/follow-up-monitoring` 现在会在同一组过滤条件下返回最小 follow-up SLA monitoring 聚合视图
- `follow_up_items` 与 `follow_up_summary` 当前已进入任务详情 contract，其他场景仍保持空数组和零值汇总
- `shift handoff` 请求现在会受控生成 1 条 seeded `handoff_receipt` draft，并关联已生成的 follow-up item
- `shift handoff` 请求现在可通过显式 `handoff-receipt/acknowledge` action 把 receipt 从 `published` 推进到 `acknowledged / acknowledged_with_exceptions`
- `shift handoff` 请求现在可通过显式 `handoff-receipt/escalate` action 把 receipt 从 `acknowledged_with_exceptions` 推进到 `escalated`
- `GET /api/v1/handoff-receipts` 现在会跨任务聚合返回 cross-shift receipt queue，并支持 `task_id / shift_id / receipt_status / receiving_role / receiving_actor_id / overdue_only / has_exceptions / escalated_only` 过滤
- `GET /api/v1/handoff-receipt-monitoring` 现在会在同一组过滤条件下返回最小 handoff receipt monitoring 聚合视图
- `handoff_receipt` 与 `handoff_receipt_summary` 当前已进入任务详情 contract，其他场景默认返回 `null` 和零值汇总
- `alert triage` 请求现在会受控生成 1 条 seeded `alert_cluster_draft`，并返回非零 `alert_triage_summary`
- `alert_cluster_drafts` 现在会推断 `source_system / line_id / triage_label / recommended_owner_role / cluster window`
- `alert triage` 现在除了 `repeated_alert_review` 外，还支持 `scada` 场景下的 `sustained_threshold_review` draft 形态
- `GET /api/v1/alert-clusters` 现在会跨任务聚合返回 `alert cluster` queue，并支持 `task_id / cluster_status / source_system / equipment_id / line_id / severity_band / triage_label / follow_up_owner_id / unaccepted_follow_up_only / follow_up_escalation_required / escalation_candidate / window_from / window_to / open_only` 过滤
- `GET /api/v1/alert-clusters` 现在会在每条 cluster item 上返回最小 `linked_follow_up` 摘要，直接暴露 follow-up 数量、接单状态、owner 和最高优先级 SLA 状态
- `GET /api/v1/alert-cluster-monitoring` 现在会在同一组过滤条件下返回最小 alert cluster monitoring 聚合视图，并复用 linked follow-up triage 过滤
- `GET /api/v1/alert-cluster-monitoring` 现在还会直接返回 `linked/unlinked/accepted/unaccepted/escalation` 五组 linked follow-up backlog 汇总字段与 `follow_up_coverage / follow_up_sla_status` bucket
- `alert_cluster_drafts` 与 `alert_triage_summary` 当前已进入任务详情 contract，其他场景仍返回空数组和零值汇总
- `planned_task.task.plan.governance` 现在会返回责任矩阵、审批策略和 fallback actions
- 审计事件可通过 `/api/v1/audit/events` 查看，也支持 `task_id / correlation_id / kind / approval_id` 过滤
- 单任务 evidence 可通过 `/api/v1/tasks/{task_id}/evidence` 查看
- 单任务 governance 可通过 `/api/v1/tasks/{task_id}/governance` 查看
- 单任务审计回放可通过 `/api/v1/tasks/{task_id}/audit-events` 查看
- `priority` 当前只接受 `routine / expedited / critical`
- `risk` 当前只接受 `low / medium / high / critical`

查询已跟踪任务：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0 | jq
```

查看跨任务 follow-up queue：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/follow-up-items?owner_id=worker_1101" | jq
```

说明：

- 第一版 queue 会跨任务聚合所有已持久化的 `follow_up_item`
- 当前支持 `task_id / source_kind / status / owner_id / owner_role / overdue_only / blocked_only / escalation_required / due_before / risk / priority` 过滤
- 默认按 `effective_sla_status -> due_at -> task_priority -> updated_at` 排序
- 当前 queue 直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 projection 表

查看高风险且需要升级的 follow-up queue：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/follow-up-items?escalation_required=true&risk=high&priority=expedited" | jq
```

查看 follow-up SLA monitoring 视图：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/follow-up-monitoring?risk=high" | jq
```

说明：

- monitor 复用 `GET /api/v1/follow-up-items` 的过滤语义
- 当前返回 `total_items / open_items / accepted_items / unaccepted_items / blocked_items / overdue_items / escalation_required_items / next_due_at`
- 当前还会返回 `source_kind / owner_role / effective_sla_status / task_risk / task_priority` 五组最小 bucket 统计
- 第一版 monitor 同样直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 dedicated projection

查看跨班次 handoff receipt queue：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/handoff-receipts?overdue_only=true" | jq
```

说明：

- 第一版 receipt queue 会跨任务聚合所有已持久化的 `shift handoff` `handoff_receipt`
- 当前支持 `task_id / shift_id / receipt_status / receiving_role / receiving_actor_id / overdue_only / has_exceptions / escalated_only` 过滤
- 默认按 `effective_status -> required_ack_by -> task_priority -> updated_at` 排序
- `effective_status` 会把超时未确认的 `published` receipt 读时收敛成 `expired`
- 当前 queue 直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 receipt projection 表

查看 handoff receipt monitoring 视图：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/handoff-receipt-monitoring?escalated_only=true" | jq
```

说明：

- monitor 复用 `GET /api/v1/handoff-receipts` 的过滤语义
- 当前返回 `total_receipts / open_receipts / acknowledged_receipts / unacknowledged_receipts / overdue_receipts / exception_receipts / escalated_receipts / next_ack_due_at`
- 当前还会返回 `effective_status / receiving_role / ack_window / task_risk / task_priority` 五组最小 bucket 统计
- `ack_window_counts` 当前按 `overdue / due_within_30m / due_within_2h / future / no_deadline` 收敛未确认 receipt 的确认窗口
- 第一版 monitor 同样直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 dedicated projection

查看跨任务 alert cluster queue：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/alert-clusters?escalation_candidate=true&open_only=true" | jq
```

说明：

- 第一版 alert cluster queue 会跨任务聚合所有已持久化的 `alert_cluster_draft`
- 当前支持 `task_id / cluster_status / source_system / equipment_id / line_id / severity_band / triage_label / follow_up_owner_id / unaccepted_follow_up_only / follow_up_escalation_required / escalation_candidate / window_from / window_to / open_only` 过滤
- 默认按 `escalation_candidate -> cluster_status -> severity_band -> active_or_recent_window -> updated_at` 排序
- 当前每个 cluster item 都会携带 `linked_follow_up.total_items / open_items / accepted_items / unaccepted_items / accepted_owner_ids / worst_effective_sla_status`
- linkage 优先匹配显式 `source_kind=alert_cluster + cluster_id`，并兼容现有单 cluster task 上的 `alert_triage` follow-up
- `follow_up_owner_id` 可按已 accepted 的实际 owner 过滤 cluster backlog
- `unaccepted_follow_up_only=true` 可只返回仍有 follow-up 未被 accepted 的 cluster
- `follow_up_escalation_required=true` 可只返回 linked follow-up 已进入升级态的 cluster
- `window_from / window_to` 当前按 cluster window overlap 语义过滤
- 当前 queue 直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 alert-cluster projection 表

查看 alert cluster monitoring 视图：

```bash
curl -sS "http://127.0.0.1:8000/api/v1/alert-cluster-monitoring?source_system=scada" | jq
```

说明：

- monitor 复用 `GET /api/v1/alert-clusters` 的过滤语义
- 当前返回 `total_clusters / open_clusters / escalation_candidate_clusters / high_severity_clusters / active_window_clusters / stale_window_clusters / next_window_end_at`
- 当前还会返回 `cluster_status / source_system / severity_band / triage_label / owner_role / window_state / task_risk / task_priority` 八组最小 bucket 统计
- 当前还会返回 `linked_follow_up_clusters / unlinked_follow_up_clusters / accepted_follow_up_clusters / unaccepted_follow_up_clusters / follow_up_escalation_clusters`
- 当前还会返回 `follow_up_coverage_counts / follow_up_sla_status_counts` 两组 linked follow-up backlog bucket
- monitor 现在也支持 `follow_up_owner_id / unaccepted_follow_up_only / follow_up_escalation_required` 这些 linked follow-up triage 过滤
- `window_state_counts` 当前按 `active / stale / future` 收敛 cluster window 所处状态
- 第一版 monitor 同样直接复用内存 / 文件 / SQLite repository 里的 `TrackedTaskState` 扫描，不额外引入 dedicated projection

接手一条 follow-up 待办：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f1/follow-up-items/fu_72c8f5d00f084e0ca8c41d4dc51a25f1_handoff_review/accept-owner \
  -H "x-correlation-id: demo-follow-up-accept-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_1101",
      "display_name": "Zhang Incoming",
      "role": "Incoming Shift Supervisor"
    },
    "note": "Incoming shift supervisor accepts remaining work ownership."
  }' | jq
```

说明：

- 该 action 只适用于任务详情中已存在的 `follow_up_item`
- 第一版只允许 `draft -> accepted`
- 如果 item 带有 `recommended_owner_role`，`actor.role` 会按该角色强校验
- `shift handoff` receipt summary 会同步更新 `unaccepted_follow_up_count`

确认交接回执：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f1/handoff-receipt/acknowledge \
  -H "x-correlation-id: demo-handoff-ack-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_1101",
      "display_name": "Zhang Incoming",
      "role": "Incoming Shift Supervisor"
    },
    "exception_note": null
  }' | jq
```

说明：

- 该 action 只适用于已有 `handoff_receipt` 的 `shift handoff` 任务
- `actor.role` 会按 `receiving_role` 强校验，当前要求 `incoming_shift_supervisor`
- 如果提供 `exception_note`，receipt 会进入 `acknowledged_with_exceptions`

升级存在异议的交接回执：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f1/handoff-receipt/escalate \
  -H "x-correlation-id: demo-handoff-escalate-001" \
  -H "Content-Type: application/json" \
  -d '{
    "actor": {
      "id": "worker_1001",
      "display_name": "Liu Supervisor",
      "role": "Production Supervisor"
    },
    "note": "Escalate to day-shift review before startup release."
  }' | jq
```

说明：

- 该 action 只允许 `acknowledged_with_exceptions -> escalated`
- `actor.role` 会按发送侧 accountable 角色强校验；当前 seeded `shift handoff` receipt 要求 `production_supervisor`
- `exception_note` 会保留在 receipt 上，便于 review / escalation 回放

批准待审批任务：

```bash
curl -sS http://127.0.0.1:8000/api/v1/tasks/72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0/approve \
  -H "x-correlation-id: demo-approve-001" \
  -H "Content-Type: application/json" \
  -d '{
    "decided_by": {
      "id": "worker_2001",
      "display_name": "Wang Safety",
      "role": "Safety Officer"
    },
    "approved": true,
    "comment": "Proceed to execution"
  }' | jq
```

说明：

- `approve` 现在会按 `planned_task.task.plan.governance.approval_strategy.required_role` 强校验 `decided_by.role`
- 当前高风险示例任务要求的审批角色是 `safety_officer`

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
- `scripts/smoke_v0_2_0.sh` 和 CI 负责主 workflow 的 smoke gate
- `scripts/smoke_v0_2_0_sandbox.sh` 提供受限环境下的 socket-free smoke 路径
- Release 通过 `v*` tag 触发

详细说明见：

- [docs/architecture.md](docs/architecture.md)
- [docs/project-charter.md](docs/project-charter.md)
- [docs/roadmap.md](docs/roadmap.md)
- [docs/release-process.md](docs/release-process.md)
- [docs/release/v0.2.0-release-notes.md](docs/release/v0.2.0-release-notes.md)
- [docs/governance/README.md](docs/governance/README.md)
- [docs/planning/README.md](docs/planning/README.md)
- [docs/progress/README.md](docs/progress/README.md)
- [docs/journal/README.md](docs/journal/README.md)
- [docs/qa/README.md](docs/qa/README.md)

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
- 制造行业 Agentic AI 场景全景与场景分层优先级基线
- 第二条候选 workflow `质量偏差隔离与处置建议` specification baseline
- 第二条候选质量 workflow 的 connector / evidence / governance / API 对齐清单 baseline
- 面向制造现场日常运营的高频、高优先级 Agentic 功能优先级地图 baseline
- 高频日常协同 workflow `班次交接摘要与待办提取` specification baseline
- 高频事件协同 workflow `产线告警聚合与异常分诊` specification baseline
- follow-up / owner / due date / SLA 通用模型对齐清单 baseline
- follow-up / SLA read model 与 query direction note baseline
- follow-up task read model implementation cut note
- follow-up task detail schema cut baseline
- seeded follow-up draft baseline for `shift handoff`
- seeded follow-up draft baseline for `alert triage`
- explicit follow-up owner acceptance action baseline for seeded task-scoped items
- cross-task follow-up owner queue baseline via `GET /api/v1/follow-up-items`
- operational triage filters for follow-up queue via `blocked_only / escalation_required / due_before / risk / priority`
- follow-up SLA monitoring baseline via `GET /api/v1/follow-up-monitoring`
- shift handoff workflow connector / evidence / SLA 对齐清单 baseline
- shift handoff receipt / acknowledgement direction note baseline
- shift handoff receipt task read model implementation cut note
- shift handoff receipt task detail schema cut baseline
- seeded handoff receipt draft baseline for `shift handoff`
- explicit handoff receipt acknowledgement action baseline for `shift handoff`
- cross-shift handoff receipt queue baseline via `GET /api/v1/handoff-receipts`
- handoff receipt monitoring baseline via `GET /api/v1/handoff-receipt-monitoring`
- alert triage workflow connector / evidence / governance / event-ingestion 对齐清单 baseline
- alert triage alert-cluster / event-ingestion direction note baseline
- alert triage task read model implementation cut note
- alert triage task detail schema cut baseline
- seeded alert cluster draft baseline for `alert triage`
- richer alert cluster source / line / window inference baseline for `alert triage`
- second `sustained_threshold_review` alert cluster draft mode baseline for `scada` triage requests
- alert cluster queue linked follow-up summary baseline for owner and SLA visibility
- alert cluster linked follow-up triage filters baseline for queue and monitoring reads
- alert cluster monitoring linked follow-up aggregates baseline for backlog coverage and SLA visibility
- quality `QMS` mock connector baseline note
- 结构化 task evidence snapshot 与任务级 evidence 查询接口
- `v0.2.0` 测试清单、发布清单与可重复 smoke script 基线
- workflow responsibility matrix 与 approval strategy 基线
- 面向受限环境的 sandbox-safe smoke 路径与项目内 `sandbox/` 运行目录
- 内存审计事件流与 `correlation_id` 贯通
- 服务层生命周期集成测试
- 基础测试
- CI / Release 基线
- 架构与项目文档基线

下一步优先级：

1. 评估把 `alert-clusters / alert-cluster-monitoring` 从 repository-scan 读层推进到 dedicated projection / backlog aging slices
2. 评估把 `alert-clusters / alert-cluster-monitoring` 里的 `linked_follow_up` 摘要、triage filters 和 monitoring aggregates 推进到 dedicated projection / backlog aging / escalation slices
3. 评估是否为 `alert-cluster-monitoring` 增加 accepted owner / owner-load 聚合维度
4. 评估把 `follow-up-monitoring` 从 repository-scan 聚合推进到 dedicated projection / backlog aging slices
5. 评估把 `handoff-receipt-monitoring` 从 repository-scan 聚合推进到 dedicated projection / aging trend slices
6. 评估把 mock `QMS` baseline 从设计说明推进到默认 registry 和 read plan 代码实现的切口

## 团队工作流

项目团队的标准工作流程已经固化在仓库中，后续需求、设计、开发、测试、发布和试运行都按这些文档执行：

- [docs/governance/team-operating-model.md](docs/governance/team-operating-model.md)
- [docs/governance/delivery-lifecycle.md](docs/governance/delivery-lifecycle.md)
- [docs/governance/governance-controls.md](docs/governance/governance-controls.md)

## 当前执行计划

当前已经把 `M1` 的执行计划细化到工作包、责任、依赖和验收层，后续实施以此为准：

- [docs/planning/m1-execution-plan.md](docs/planning/m1-execution-plan.md)
