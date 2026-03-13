# M1 Sandbox-Safe Smoke Baseline Kickoff

## 日期

2026-03-12

## 同步目的

为受限执行环境补上一条不依赖本地 TCP 监听的 smoke 路径，同时把临时验证数据统一收敛到项目内的 `sandbox/` 目录。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`sandbox-safe smoke validation`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：开始本阶段前包含未提交的 approval role enforcement 变更；本阶段在其上继续推进

## 上一阶段完成基线

上一阶段已完成并推送：

- approval role enforcement
- `403 Forbidden` 审批角色错误映射
- README / QA / smoke 示例同步

## 本阶段目标

1. 找到不依赖本地 socket 的 sandbox-safe smoke 方案。
2. 在文件模式下保留重启回读和 HTTP 路由级验证价值。
3. 把默认临时目录迁移到项目内 `sandbox/`。
4. 为受限环境补充 Makefile、脚本和文档入口。

## 本阶段交付边界

本阶段计划交付：

- `scripts/smoke_v0_2_0_sandbox.sh`
- in-process file-backed smoke test
- `make smoke-sandbox`
- `make release-check-sandbox`
- `sandbox/` 目录约定与 `.gitignore` 规则

本阶段暂不交付：

- 替代正式 release gate 的新流程
- 网络层代理或端口转发方案
- 独立的 smoke binary

## 风险与注意事项

- sandbox-safe smoke 不能误导成“真实 listener 已验证”
- 仍需保留真实 HTTP smoke 作为发布门禁
- 项目内 `sandbox/` 必须避免污染 Git 历史

## 进入本阶段的理由

如果项目只能在无约束的本机环境里验证，就很难在沙箱、受控 runner 或受限自动化环境中持续推进。补齐 socket-free 路径，能让验证能力更稳健，而不是依赖一类特定运行条件。

## 本阶段完成结果

- 已交付 `scripts/smoke_v0_2_0_sandbox.sh`
- 已交付进程内 file-backed smoke test
- 已交付 `make smoke-sandbox` 与 `make release-check-sandbox`
- 默认 smoke 数据目录已切到项目内 `sandbox/`

## 实现摘要

本阶段没有尝试绕过沙箱限制去强行起 listener，而是把验证重心放在进程内 HTTP 路由调用和文件持久化回读上。这样既保留了 API 级语义验证，也避免了受限环境下最容易失败的端口监听动作。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

覆盖结果：

- `sandbox/` 下文件模式可用
- 重启前后任务、evidence、governance 可回读
- 审批角色强校验在 sandbox-safe 路径中仍成立
- 完整主链不依赖本地 TCP 监听即可验证

## 阶段收口结论

项目现在既保留了正式的真实 listener smoke，也拥有了受限环境可执行的 sandbox-safe smoke。下一步可以继续收口 `v0.2.0` release note，而不再被本地 socket 限制卡住。
