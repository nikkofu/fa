# FA Roadmap

## 2026-03-11 Baseline

版本目标：`v0.1.0`

已建立：

- Rust workspace
- 领域模型
- 编排器与模式选择
- API skeleton
- 文档、CI、Release 基线

## v0.2.0

发布日期：2026-03-13

当前状态：`Released`

发布结论：`M1` 的 `v0.2.0` 交付已经完成，版本已进入 GitHub release 发布流程。

已完成的子项：

- 任务状态机
- 审批状态机
- `connector` trait
- `audit` trait
- request / trace correlation id
- mock `MES` / `CMMS` read-only connector
- `intake / get / approve / execute / complete / fail` 生命周期主路径
- 服务层生命周期集成测试
- task repository abstraction
- rejected task resubmission loop
- local persistence baseline for task and audit storage
- audit replay and filtered audit query baseline
- sqlite-backed local persistence baseline
- pilot workflow candidate evaluation baseline
- first pilot workflow specification baseline
- structured task evidence snapshot baseline
- v0.2.0 QA and release readiness baseline
- workflow governance matrix and approval strategy baseline
- approval role enforcement for governed tasks
- sandbox-safe smoke validation path for restricted environments
- manufacturing Agentic AI scenario landscape baseline
- quality deviation candidate workflow specification baseline
- quality workflow alignment checklist baseline
- high-frequency Agentic function priority map baseline
- shift handoff workflow specification baseline
- alert triage workflow specification baseline
- follow-up and SLA model alignment checklist baseline
- follow-up and SLA read model and query direction note baseline
- follow-up task read model implementation cut note baseline
- follow-up task detail schema cut baseline
- seeded follow-up draft baseline for shift handoff
- seeded follow-up draft baseline for alert triage
- shift handoff workflow alignment checklist baseline
- shift handoff receipt and acknowledgement direction note baseline
- shift handoff receipt task read model implementation cut note baseline
- shift handoff receipt task detail schema cut baseline
- seeded handoff receipt draft baseline for shift handoff
- explicit handoff receipt acknowledgement action baseline for shift handoff
- alert triage workflow alignment checklist baseline
- alert triage alert-cluster and event-ingestion direction note baseline
- alert triage task read model implementation cut note baseline
- alert triage task detail schema cut baseline
- seeded alert cluster draft baseline for alert triage
- follow-up owner acceptance action baseline
- cross-task follow-up queue triage filter baseline
- follow-up SLA monitoring baseline
- cross-shift handoff receipt queue and monitoring baseline
- cross-task alert cluster queue and monitoring baseline
- linked follow-up summary, triage filters, and monitoring aggregates on alert cluster reads
- quality qms mock connector baseline note

计划内容：

- 任务状态机
- 审批状态机
- `connector` trait
- `audit` trait
- request / trace correlation id
- 开始接入只读 MES / CMMS mock connector

验收条件：

- 任务不再只有规划结果，而具备生命周期
- connector 可替换，不与具体供应商绑定
- 生命周期 API 主路径可通过服务层测试验证
- task state storage 可通过 repository abstraction 替换而不改 API 主路径
- rejected work 可重新发起审批而不必重建任务
- 本地文件持久化可支撑重启后的任务与审计回读
- 审计事件可按任务、相关链路和事件类型回放与过滤
- SQLite 本地数据库可承接任务与审计的耐久存储
- 首条制造 pilot workflow 已明确业务边界、审批角色、SOP 影响与回退策略
- 任务证据已经可以作为结构化快照进入 API、持久化和任务级查询
- `v0.2.0` 已具备测试清单、发布清单和可重复 smoke gate 基线
- 任务计划已具备责任矩阵、审批策略和 fallback actions 输出
- 受限环境下也有可执行的 sandbox-safe smoke 路径
- 第二条质量候选 workflow 已具备与当前 connector / evidence / governance / API 的对齐清单
- 已识别一组更适合先产品化的高频、高优先级制造 Agentic 功能
- 已为首个高频日常协同功能输出正式 workflow specification baseline
- 已为首个高频事件协同功能输出正式 workflow specification baseline
- 已为 follow-up / owner / due date / SLA 通用协同模型输出正式对齐清单
- 已为 follow-up / SLA 输出 task-scoped read model 与 cross-task query direction note
- 已为 `follow_up_items` 进入 `tasks/{task_id}` 输出 implementation cut note
- 已为 `follow_up_items / follow_up_summary` 完成兼容旧 JSON 的 task detail schema cut
- 已为 `shift handoff` 请求生成首条 seeded `follow_up_item` draft
- 已为 `alert triage` 请求生成第二条高频 seeded `follow_up_item` draft
- 已为 `follow_up_items` 补最小 `follow-up-items/{follow_up_id}/accept-owner` action
- 已为 `follow_up_items` 补最小 `GET /api/v1/follow-up-items` cross-task owner queue
- 已为 `GET /api/v1/follow-up-items` 补最小 `blocked / escalation / due_before / risk / priority` triage filters
- 已为 `follow_up_items` 补最小 `GET /api/v1/follow-up-monitoring` SLA monitoring 视图
- 已为 shift handoff workflow 输出 connector / evidence / SLA 对齐清单
- 已为 shift handoff workflow 输出 receipt / acknowledgement direction note
- 已为 `handoff_receipt` 进入 `tasks/{task_id}` 输出 implementation cut note
- 已为 `handoff_receipt / handoff_receipt_summary` 完成兼容旧 JSON 的 task detail schema cut
- 已为 `shift handoff` 请求生成首条 seeded `handoff_receipt` draft
- 已为 `shift handoff` 请求补最小 `handoff-receipt/acknowledge` action
- 已为 `shift handoff` receipt 补最小 `handoff-receipt/escalate` action
- 已为 `shift handoff` receipt 补最小 `GET /api/v1/handoff-receipts` cross-shift queue
- 已为 `shift handoff` receipt 补最小 `GET /api/v1/handoff-receipt-monitoring` monitoring 视图
- 已为 alert triage workflow 输出 connector / evidence / governance / event-ingestion 对齐清单
- 已为 alert triage workflow 输出 alert cluster / event-ingestion direction note
- 已为 `alert_cluster_drafts / alert_triage_summary` 完成兼容旧 JSON 的 task detail schema cut
- 已为 `alert_cluster_drafts` 进入 `tasks/{task_id}` 输出 implementation cut note
- 已为 `alert triage` 请求生成首条 seeded `alert_cluster_draft`
- 已为 `alert_cluster_drafts` 补最小 richer source / line / window inference
- 已为 `alert triage` 补第二类 `sustained_threshold_review` cluster draft 形态
- 已为 `alert cluster` 补最小 `GET /api/v1/alert-clusters` cross-task queue
- 已为 `alert cluster` 补最小 `GET /api/v1/alert-cluster-monitoring` monitoring 视图
- 已为 `GET /api/v1/alert-clusters` 补最小 linked follow-up 摘要联动
- 已为 quality workflow 输出 qms mock connector baseline note

新增验证基线：

- `tasks/intake` 与 `tasks/{task_id}` 现在都会返回 `follow_up_items / follow_up_summary`
- file / SQLite repository 已验证可兼容持久化与回读新的 follow-up task detail 字段
- `shift handoff` 请求现在可返回非空 `follow_up_items / follow_up_summary`
- `alert triage` 请求现在可返回非空 `follow_up_items / follow_up_summary`
- seeded `follow_up_item` 现在可通过显式 action 从 `draft` 进入 `accepted`
- `GET /api/v1/follow-up-items` 现在可跨任务返回 follow-up owner queue
- `GET /api/v1/follow-up-items` 现在支持 `owner_id / source_kind` 等最小过滤
- `GET /api/v1/follow-up-items` 现在支持 `blocked_only / escalation_required / due_before / risk / priority` 等运营 triage 过滤
- `GET /api/v1/follow-up-monitoring` 现在可返回最小 follow-up SLA monitoring 聚合视图
- `GET /api/v1/follow-up-monitoring` 现在支持与 `GET /api/v1/follow-up-items` 一致的过滤语义
- `shift handoff` task detail 会在 owner acceptance 后同步收敛 `handoff_receipt_summary.unaccepted_follow_up_count`
- `tasks/intake` 与 `tasks/{task_id}` 现在都会返回 `handoff_receipt / handoff_receipt_summary`
- file / SQLite repository 已验证可兼容持久化与回读新的 handoff receipt task detail 字段
- `shift handoff` 请求现在可返回非空 `handoff_receipt / handoff_receipt_summary`
- `shift handoff` receipt 现在可通过显式 action 从 `published` 进入 `acknowledged / acknowledged_with_exceptions`
- `shift handoff` receipt 现在可通过显式 action 从 `acknowledged_with_exceptions` 进入 `escalated`
- `GET /api/v1/handoff-receipts` 现在可跨任务返回 cross-shift receipt queue
- `GET /api/v1/handoff-receipts` 现在支持 `shift_id / receipt_status / receiving_role / receiving_actor_id / overdue_only / has_exceptions / escalated_only` 过滤
- `GET /api/v1/handoff-receipt-monitoring` 现在可返回最小 handoff receipt monitoring 聚合视图
- `GET /api/v1/handoff-receipt-monitoring` 现在支持与 `GET /api/v1/handoff-receipts` 一致的过滤语义
- `tasks/intake` 与 `tasks/{task_id}` 现在都会返回 `alert_cluster_drafts / alert_triage_summary`
- file / SQLite repository 已验证可兼容持久化与回读新的 alert cluster task detail 字段
- `alert triage` 请求现在可返回非空 `alert_cluster_drafts / alert_triage_summary`
- `alert_cluster_drafts` 现在可推断 `source_system / line_id / triage_label / recommended_owner_role / cluster window`
- `scada` 阈值类告警现在可生成 `sustained_threshold_review` cluster draft 与 `maintenance_engineer` 路由建议
- `GET /api/v1/alert-clusters` 现在可跨任务返回 alert cluster queue
- `GET /api/v1/alert-clusters` 现在支持 `cluster_status / source_system / equipment_id / line_id / severity_band / triage_label / follow_up_owner_id / unaccepted_follow_up_only / follow_up_escalation_required / escalation_candidate / window_from / window_to / open_only` 过滤
- `GET /api/v1/alert-clusters` 现在会在每条 cluster item 上返回 `linked_follow_up` 摘要，暴露 total/open/accepted/unaccepted/accepted_owner_ids/worst_effective_sla_status
- `GET /api/v1/alert-cluster-monitoring` 现在可返回最小 alert cluster monitoring 聚合视图
- `GET /api/v1/alert-cluster-monitoring` 现在支持与 `GET /api/v1/alert-clusters` 一致的过滤语义，包括 linked follow-up triage 过滤
- `GET /api/v1/alert-cluster-monitoring` 现在会返回 linked/unlinked/accepted/unaccepted/escalation 五组 linked follow-up backlog 汇总字段，以及 `follow_up_coverage / follow_up_sla_status` bucket

执行计划：

- [planning/m1-execution-plan.md](planning/m1-execution-plan.md)
- [planning/pilot-workflow-candidates.md](planning/pilot-workflow-candidates.md)
- [planning/manufacturing-agentic-ai-scenario-landscape.md](planning/manufacturing-agentic-ai-scenario-landscape.md)
- [planning/high-frequency-agentic-function-priority-map.md](planning/high-frequency-agentic-function-priority-map.md)

## v0.3.0

目标日期：2026-04-18

计划内容：

- LLM provider abstraction
- evidence store abstraction
- prompt / tool / policy boundary 分层
- 高风险任务审批链

验收条件：

- 能基于外部知识与业务上下文完成一次带证据的诊断规划

## v0.4.0

目标日期：2026-05-15

计划内容：

- Pilot workflow 端到端打通
- 审计日志与回放视图
- UAT 测试脚本
- 部署清单与试运行指南

验收条件：

- 支持至少 1 条制造试运行场景闭环

## v1.0.0

前提：

- 至少完成一次受控试运行
- 发布、回退、审计、权限与 KPI 管理完整
- 已验证真实业务价值和运行稳定性
