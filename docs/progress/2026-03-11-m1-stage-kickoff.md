# M1 Stage Kickoff Status

## 日期

2026-03-11

## 同步目的

在进入 `M1` 下一阶段任务前，正式同步当前项目状态、版本状态、仓库状态和下一步工作边界。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 阶段重点：
  - 生命周期主链
  - 只读 connector mock
  - 审计基线
  - 任务生命周期 API

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 阶段启动基线 commit：`fcb9e77`
- 阶段启动时本地分支与 `origin/main` 同步

## 当前已完成能力

- Rust workspace 与服务骨架
- 制造领域模型
- 任务规划与模式选择
- 任务生命周期模型
- 审批生命周期模型
- mock `MES` / mock `CMMS` connector
- in-memory audit sink
- `intake / get / approve / execute / complete / fail / audit` API 主路径
- 服务层生命周期集成测试

## 当前未完成能力

- 可替换持久化 task repository
- 审批与执行的更多异常路径
- 试运行工作流规格冻结

## 当前阶段结论

`M1` 已经从“概念与计划阶段”进入“可运行原型阶段”。系统现在可以：

1. 接收制造任务。
2. 读取 mock 企业系统上下文。
3. 生成任务与审批记录。
4. 驱动审批与执行主状态流。
5. 通过 correlation id 跟踪审计事件。
6. 通过服务层集成测试验证主路径稳定性。

这说明平台已具备最小编排骨架，不再只是静态 demo。

## 质量与运行态验证

本阶段已完成以下验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 在 `127.0.0.1:8000` 上完成本地 smoke test

运行态已验证的主路径：

1. `intake -> approve -> execute -> complete`
2. `intake -> approve -> execute -> fail`
3. `/api/v1/audit/events` 可回看关联审计记录

运行态发现并确认的接口契约：

- `priority` 不是自由文本，当前必须使用 `routine / expedited / critical`
- 这类契约约束已经在服务层生效，说明原型不只是“能跑”，而是开始具备稳定接口边界

## 下一阶段任务

下一阶段的实现重点：

1. 将 in-memory task store 演进为可替换 repository。
2. 扩展审批、执行、失败的异常路径。
3. 为下一条试运行 workflow 冻结规格输入。

## 风险提醒

- 当前 task store 仍是内存实现，重启即失。
- 当前 connector 是 mock，不代表真实供应商接口复杂度。
- 当前 execute 仍是 stub，不代表真实设备/系统写入流程。

## 进入下一阶段的理由

当前基础已经足够支持继续扩展，而不会陷入“边界不清导致返工”的状态。继续推进 `complete / fail` 与集成测试，是当前最合理的投资。
