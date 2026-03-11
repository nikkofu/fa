# Delivery Lifecycle

## 1. 目的

本文件定义 FA 项目从需求进入到发布关闭的端到端工作流，确保软件工程活动与 PMP 管理动作对齐。

## 2. 工作项分层

| 层级 | 含义 | 典型负责人 |
| --- | --- | --- |
| Initiative | 跨季度或跨里程碑的业务目标 | Sponsor / PO |
| Epic | 可独立评估价值的一组能力 | PO / Architect |
| Feature | 一项面向用户或系统的能力 | PO / Eng Lead |
| Story / Task | 可在一个迭代内完成的工作单元 | Eng Lead / Engineer |
| Bug | 偏离预期行为的缺陷 | QA Lead / Engineer |
| Risk | 可能影响目标达成的不确定事项 | Delivery Lead |
| Change Request | 对范围、时间、资源、架构基线的受控变更 | Delivery Lead / PO |

## 3. 标准状态流

标准工作项使用以下状态：

1. `Proposed`
2. `Discovery`
3. `Qualified`
4. `Planned`
5. `Ready for Dev`
6. `In Progress`
7. `In Review`
8. `In Test`
9. `Ready for UAT`
10. `Ready for Release`
11. `Released`
12. `Closed`

特殊状态：

- `Blocked`
- `Rejected`
- `Deferred`

## 4. 端到端阶段

### 4.1 Stage 0: Intake

目标：

- 收集新需求、缺陷、风险或变更请求。

输入：

- 业务反馈
- 试运行问题
- 路线图目标
- 技术债

活动：

- 创建 issue
- 标注类型、来源、影响范围、初步优先级
- 指定 owner 进入 Discovery

输出：

- 已编号的工作项

入口条件：

- 有明确的触发背景

出口条件：

- issue 信息足够支撑 Discovery

### 4.2 Stage 1: Discovery

目标：

- 澄清业务问题、用户角色、系统影响和风险。

活动：

- 明确业务场景和使用者
- 明确涉及的系统、设备、审批角色
- 判断是否需要设计评审、风险评审、POC
- 初步估算复杂度和依赖

输出：

- 问题定义
- 业务价值说明
- 初步验收标准
- 依赖与风险草案

出口条件：

- 能判断是否值得进入 Qualified

### 4.3 Stage 2: Qualification

目标：

- 判断该项工作是否进入计划池，并满足最小可计划条件。

必须明确：

- Why now
- 业务价值或缺陷影响
- 范围边界
- 验收标准
- 风险级别
- 是否影响架构、审批、设备、安全、外部系统

输出：

- 可排期的 feature / bug / change request

出口条件：

- 满足 Definition of Ready

### 4.4 Stage 3: Planning

目标：

- 将需求纳入版本、里程碑和短周期迭代计划。

活动：

- 分解 WBS
- 确认 owner 和依赖
- 对齐目标版本
- 拆出设计、开发、测试、文档、发布、试运行准备任务

输出：

- 已排期工作项
- 版本关联
- 责任分配

出口条件：

- 已进入具体迭代或周计划

### 4.5 Stage 4: Solution design

目标：

- 对关键需求先完成设计，避免开发时边做边猜。

触发条件：

- 影响架构边界
- 新增 connector / provider
- 新审批链
- 新 agent 模式
- 影响设备动作或业务写入

活动：

- 设计评审
- 更新架构文档或 ADR
- 确认测试策略和安全边界

输出：

- 设计结论
- ADR 或设计文档
- 实施拆解

出口条件：

- 风险和设计关键点已明确

### 4.6 Stage 5: Implementation

目标：

- 按计划完成编码、单测、文档和自测。

活动：

- feature branch 开发
- 增量提交
- 本地 `fmt / clippy / test`
- 同步更新相关文档

输出：

- 可审阅 PR

出口条件：

- 代码、测试、文档齐备

### 4.7 Stage 6: Review

目标：

- 完成代码评审与设计一致性校验。

活动：

- PR review
- 架构点检查
- 风险与回退说明确认
- CI 通过

输出：

- 可合并 PR

出口条件：

- 审核通过
- 所有关联讨论关闭或明确处理方式

### 4.8 Stage 7: Verification

目标：

- 在测试层确认功能行为、回归结果和非功能要求。

活动：

- 集成测试
- 回归测试
- 接口测试
- 缺陷记录与修复验证

输出：

- 测试结果
- 缺陷清单
- 发布建议

出口条件：

- 满足测试退出准则

### 4.9 Stage 8: UAT and pilot readiness

目标：

- 确认业务上可以受控试用或上线。

活动：

- PO / SME 演示与验收
- Pilot 环境检查
- SOP / 审批 / 回退策略确认

输出：

- UAT 结论
- Go / No-Go 建议

出口条件：

- 满足试运行或发布门禁

### 4.10 Stage 9: Release

目标：

- 进行版本发布和必要的发布说明同步。

活动：

- 更新 changelog
- 打 tag
- 触发 release workflow
- 记录版本、风险、回退方案

输出：

- 已发布版本
- Release note

出口条件：

- 版本可追溯

### 4.11 Stage 10: Hypercare and closure

目标：

- 跟踪发布后稳定性并完成关闭。

活动：

- 观察缺陷与运行指标
- 处理高优先级问题
- 复盘需求命中度和实施偏差

输出：

- 关闭确认
- 复盘结论

出口条件：

- 工作项满足关闭条件或转入后续 issue

## 5. Definition of Ready

工作项进入 `Ready for Dev` 前必须满足：

- 有 issue 编号
- 有业务背景和问题描述
- 有验收标准
- 有 owner
- 有目标版本或迭代
- 已识别依赖和主要风险
- 涉及设计变更时已有设计结论或评审安排

额外要求：

- 涉及设备动作、业务写入、审批、安全策略的项，必须定义回退思路和审批角色。

## 6. Definition of Done

工作项进入 `Closed` 前必须满足：

- 代码已合并到主分支
- 对应测试已通过
- 必要文档已更新
- 验收标准已逐条确认
- 缺陷和已知限制已记录
- 影响发布时已进入 changelog 或 release note

若是试运行相关项，还必须满足：

- Pilot owner 已知悉结果
- 回退策略已验证或明确定义

## 7. 缺陷工作流

缺陷按严重级别处理：

| 严重级别 | 定义 | 目标处理方式 |
| --- | --- | --- |
| Sev-1 | 阻断核心流程、发布不可接受、试运行中断 | 立即升级，进入 war room |
| Sev-2 | 核心能力受损但存在临时绕行 | 当前版本优先处理 |
| Sev-3 | 非核心问题或边缘功能异常 | 纳入后续迭代 |
| Sev-4 | 轻微问题、文案或体验瑕疵 | 视资源安排 |

## 8. 变更请求工作流

以下事项必须建立 `Change Request`：

- 影响里程碑日期
- 影响试运行范围
- 影响预算或关键资源
- 引入新的外部依赖或关键架构变化
- 新增高风险设备或业务写入路径

处理步骤：

1. 提交 change request issue。
2. 评估影响范围、成本、进度、风险和替代方案。
3. 由 CCB 或授权人裁定。
4. 更新 roadmap、计划和治理文档。
5. 回写执行结果。

## 9. 推荐迭代节奏

对于当前 0-1 阶段，采用：

- 一周一个 delivery cycle
- 每周承诺少量高价值项
- 每两周做一次里程碑滚动回顾

理由：

- 团队规模尚小，过长 sprint 会掩盖风险。
- 平台仍在探索期，需要快速发现假设错误。

## 10. 项目板列建议

建议 GitHub Project 至少包含以下列：

1. `Inbox`
2. `Discovery`
3. `Ready`
4. `In Progress`
5. `Review`
6. `Test`
7. `UAT / Release`
8. `Done`
9. `Blocked`

## 11. 与代码仓库的绑定规则

- 每个 feature branch 命名建议包含 issue 编号。
- 每个 PR 必须说明测试证据、文档影响、风险与回退。
- 每个版本必须可追溯到 milestone、issue 和 release tag。
