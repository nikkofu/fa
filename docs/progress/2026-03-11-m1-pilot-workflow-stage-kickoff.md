# M1 Pilot Workflow Definition Stage Kickoff

## 日期

2026-03-11

## 同步目的

在完成 SQLite 持久化基线后，进入第一条制造 pilot workflow 的定义阶段。此阶段的目标不是继续泛化平台能力，而是把“要在哪条真实业务路径上试运行”明确下来，并写成后续实现、测试、审批和试运行可以共同使用的正式输入。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`pilot workflow definition`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`34554f1`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- SQLite-backed task repository
- SQLite-backed audit store
- `FA_SQLITE_DB_PATH` 运行时注入
- SQLite 模式下真实 HTTP 持久化与回放验证

## 本阶段目标

本阶段目标：

1. 形成至少 3 条制造 pilot workflow 候选。
2. 按业务价值、风险、系统依赖和试运行可控性进行比较。
3. 选定第一条推荐 workflow。
4. 产出第一版 workflow spec，明确角色、步骤、审批点、证据源和回退策略。

## 本阶段交付边界

本阶段计划交付：

- pilot workflow 候选比较矩阵
- 第一条推荐 workflow 结论
- 第一版 workflow specification
- README / roadmap / planning / journal / progress 同步

本阶段暂不交付：

- 新 API 端点
- 真实企业系统写入
- 设备级自动闭环控制

## 风险与注意事项

- pilot workflow 不能选成“看起来很酷但试运行过大”的流程
- 必须优先选择高价值、低写风险、审批边界清晰的场景
- workflow spec 必须与当前平台能力对齐，明确哪些已支持、哪些尚未支持

## 进入本阶段的理由

如果不尽快冻结第一条 pilot workflow，平台就会持续在“技术能力越来越多，但试运行目标越来越模糊”的状态里消耗。先定义 workflow，不是降低创新，而是把创新压到真实业务路径上。
