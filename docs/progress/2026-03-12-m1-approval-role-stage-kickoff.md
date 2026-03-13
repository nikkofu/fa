# M1 Approval Role Enforcement Kickoff

## 日期

2026-03-12

## 同步目的

在 governance matrix 和 approval strategy 已经可以进入 API 输出之后，继续把审批责任边界从“可查看”推进成“可执行约束”，避免高风险任务被不匹配的角色直接放行。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`approval role enforcement`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：开始本阶段前与 `origin/main` 同步；当前包含本阶段本地变更待提交

## 上一阶段完成基线

上一阶段已完成并推送：

- governance responsibility matrix 输出
- approval strategy 输出
- `GET /api/v1/tasks/{task_id}/governance`

## 本阶段目标

1. 对审批请求执行 `required_role` 强校验。
2. 让错误角色审批在领域层和服务层都被拒绝。
3. 修正 smoke、README 和 QA 文档中的审批角色示例。
4. 保证主 workflow 与重启回读不受破坏。

## 本阶段交付边界

本阶段计划交付：

- `ApprovalRoleMismatch`
- 审批角色强校验
- `403 Forbidden` 服务层错误映射
- 单元测试、服务层测试和 smoke 更新
- progress / journal / changelog / README 同步

本阶段暂不交付：

- 多级审批链
- escalation role 的显式升级流程
- 组织级权限系统

## 风险与注意事项

- 角色比较必须稳定，不能被大小写或空格差异绕过
- 新校验不能破坏现有 approval / resubmit / execute 主链
- 示例和脚本必须跟着治理策略一起更新，否则仓库会继续传播错误用法

## 进入本阶段的理由

如果系统只会告诉你“应该由谁审批”，却不能在接口层拒绝错误角色，那么 governance 仍然只是展示信息，而不是治理约束。把 required role 真正执行起来，才说明平台开始尊重组织责任边界。

## 本阶段完成结果

- 已交付 `ApprovalRoleMismatch`
- `approve` / `reject` 已按 `required_role` 强校验审批人角色
- 审批角色不匹配时，HTTP 接口返回 `403 Forbidden`
- README、smoke、pilot workflow spec 和 QA 文档已同步

## 实现摘要

本阶段把审批角色校验放在 `ApprovalRecord` 领域对象里，而不是只放在 API 层。这样做让文件模式、SQLite 模式、服务层测试和后续其它入口都能共享同一条治理规则，不会因为接入方式不同出现行为分叉。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash scripts/smoke_v0_2_0.sh`

覆盖结果：

- 错误审批角色会被拒绝
- 高风险任务仍要求 `safety_officer`
- 匹配角色的 `approve -> execute -> complete` 主链成立
- 文件模式重启回读仍成立

真实运行记录：

- 地址：`127.0.0.1:8000`
- 模式：`FA_DATA_DIR`
- 临时数据目录：`/tmp/fa-v0.2.0-smoke-1511030520`

## 阶段收口结论

治理信息现在不仅能被读出来，也已经开始真正限制审批动作。下一步更适合把重点放在最终 `v0.2.0` release note 整理和更完整的审批升级路径设计上。
