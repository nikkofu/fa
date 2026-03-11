# M1 Governance Matrix Baseline Kickoff

## 日期

2026-03-12

## 同步目的

在 release readiness 基线已经建立后，继续收紧首条 pilot workflow 的治理表达，把“责任矩阵”和“审批策略”从规范文字推进为真实 API 输出和任务可回读对象。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`workflow governance matrix and approval strategy`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`f29c564`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- `docs/qa` 基线
- 可重复执行的 `v0.2.0` smoke script
- CI / release workflow smoke gate

## 本阶段目标

1. 为任务计划引入结构化 governance 对象。
2. 输出 responsibility matrix、approval strategy 和 fallback actions。
3. 增加任务级 governance 查询接口。
4. 把 governance 一并纳入 smoke、测试和文档。

## 本阶段交付边界

本阶段计划交付：

- `WorkflowGovernance`
- `ResponsibilityAssignment`
- `ApprovalStrategy`
- `GET /api/v1/tasks/{task_id}/governance`
- 测试、smoke、文档同步

本阶段暂不交付：

- 审批角色强校验
- 多级审批链
- SLA / 超时 / 过期治理自动化

## 风险与注意事项

- governance 表达必须贴着当前 workflow 能力，不做空泛管理模型
- 新字段必须兼容已有文件和 SQLite 持久化内容
- 不应为了治理表达破坏当前 intake / approval / execute 主链

## 进入本阶段的理由

如果 responsibility matrix 和 approval strategy 只写在文档里，系统仍然很难向业务方证明自己真正理解“谁负责、谁批准、谁接管”。把治理信息变成系统输出，才能让试运行和审批沟通更具体。

## 本阶段完成结果

- 已交付 `WorkflowGovernance`
- 已交付 `ResponsibilityAssignment`
- 已交付 `ApprovalStrategy`
- `planned_task.task.plan.governance` 已进入任务计划输出
- 已交付 `GET /api/v1/tasks/{task_id}/governance`

## 实现摘要

本阶段把 workflow 的治理表达直接绑定在 `ExecutionPlan` 上，而不是再引入一个脱离任务主链的配置系统。这样做让 responsibility matrix、approval strategy 和 fallback actions 可以天然进入 task details、文件持久化、SQLite 持久化和 smoke 验证路径。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash scripts/smoke_v0_2_0.sh`

smoke 覆盖结果：

- `intake` 返回 governance
- `GET /api/v1/tasks/{task_id}/governance` 成立
- 重启后 governance 可随任务一起回读
- 主 workflow 与 audit query 仍保持成立

真实运行记录：

- 地址：`127.0.0.1:8000`
- 模式：`FA_DATA_DIR`
- 临时数据目录：`/tmp/fa-v0.2.0-smoke-2364032436`

## 阶段收口结论

治理矩阵和审批策略现在已经不只是文档上的治理要求，而是任务 API 可以直接输出和回读的对象。下一阶段应把重点放在审批角色强校验和最终 `v0.2.0` release note 准备上。
