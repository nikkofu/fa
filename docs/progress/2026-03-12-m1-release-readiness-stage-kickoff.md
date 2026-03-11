# M1 Release Readiness Baseline Kickoff

## 日期

2026-03-12

## 同步目的

在 lifecycle、persistence、audit、evidence 和 pilot workflow 基线已经成形后，进入 `v0.2.0` 的发布准备阶段。这个阶段的重点是把“可运行”推进成“可验证、可发布、可试运行准备”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`release readiness baseline`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`d9be6a3`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- 结构化 task evidence snapshot
- 任务级 evidence 查询 API
- evidence 随任务状态持久化

## 本阶段目标

1. 固化 `v0.2.0` 测试清单与发布清单。
2. 增加可重复执行的本地 smoke script。
3. 把 smoke script 接入 Makefile 和 CI / release workflow。
4. 记录手工验证证据与阶段结论。

## 本阶段交付边界

本阶段计划交付：

- `docs/qa` 基线
- `scripts/smoke_v0_2_0.sh`
- `make smoke` / `make release-check`
- CI / release workflow smoke gate
- progress / journal / changelog / README 同步

本阶段暂不交付：

- 正式 `v0.2.0` tag
- UAT 结果
- 试运行工厂环境部署

## 风险与注意事项

- smoke script 必须验证真实 HTTP 主链，而不是只调用健康检查
- 不应为了发布准备而破坏当前默认端口和运行约定
- QA 资产要能被团队直接复用，而不是只服务单次演示

## 进入本阶段的理由

如果没有可重复执行的验证基线，`v0.2.0` 即使功能看起来齐了，也很难真正进入受控发布。发布准备不是附属工作，而是把工程能力变成可交付能力的必要步骤。

## 本阶段完成结果

- 已交付 `docs/qa` 基线目录
- 已交付 `v0.2.0` 测试清单、发布清单、手工验证记录
- 已交付 `scripts/smoke_v0_2_0.sh`
- 已交付 `make smoke` 与 `make release-check`
- CI / release workflow 已增加 workflow smoke gate

## 实现摘要

本阶段把“发布准备”从零散动作沉淀成仓库资产：

- 测试与发布要求进入 `docs/qa`
- 主 workflow 验证进入可重复执行脚本
- 本地运行、CI、release workflow 三个层面开始共用同一条 smoke path

## 验证记录

已完成验证：

- `bash scripts/smoke_v0_2_0.sh`

smoke 覆盖结果：

- `intake` 返回 `awaiting_approval`
- `evidence` 已出现在 intake 和 task 查询结果中
- `GET /api/v1/tasks/{task_id}/evidence` 成立
- 文件模式重启后任务与 evidence 可回读
- `approve -> execute -> complete` 主链成立
- 任务级审计回放与按 `correlation_id` 查询成立

真实运行记录：

- 地址：`127.0.0.1:8000`
- 模式：`FA_DATA_DIR`
- 临时数据目录：`/tmp/fa-v0.2.0-smoke-483818043`

## 阶段收口结论

`v0.2.0` 已经具备更像真实版本的发布准备形态，而不只是“功能加总”。下一阶段应继续收紧 pilot workflow 的角色责任矩阵、审批策略表达和最终 release note 准备。
