# M1 Execution Plan

## 1. 计划目的

本计划将 [roadmap.md](/Users/admin/Documents/WORK/ai/fa/docs/roadmap.md) 中 `v0.2.0` 和 [project-charter.md](/Users/admin/Documents/WORK/ai/fa/docs/project-charter.md) 中 `M1: Core orchestration prototype` 细化为一份可执行计划，用于指导接下来 2 到 4 周的实际工作。

本计划的目标不是覆盖所有制造场景，而是建立第一条可验证的“受治理编排底座”，让平台从“能规划”走向“能管理任务生命周期、审批链和只读集成原型”。

## 2. 计划周期

### 2.1 时间窗口

- Planning baseline date: `2026-03-11`
- Preparation window: `2026-03-11` 至 `2026-03-20`
- M1 execution window: `2026-03-23` 至 `2026-04-10`
- `v0.2.0` target release: `2026-03-31`
- M1 closeout review target: `2026-04-10`

### 2.2 交付节奏

- 每周一：计划确认与承诺
- 每工作日：daily standup
- 每周一次：架构评审
- 每周一次：RAID / change review
- 每周一次：质量与发布准备检查

## 3. M1 目标

### 3.1 业务目标

- 让一条制造异常处理类工作流具备从任务创建、规划、审批门控到执行跟踪的基础骨架。
- 为后续接入真实 ERP / MES / CMMS 奠定 connector 抽象和审计边界。
- 为试运行选择一条低风险但高价值的 pilot workflow。

### 3.2 技术目标

- 引入任务状态机与审批状态机。
- 建立 `connector` trait 和 `audit` trait。
- 引入 request / trace correlation id。
- 提供 MES / CMMS 的 mock 只读连接器。
- 完成第一版 pilot workflow 定义文档。

### 3.3 管理目标

- 将 `M1` 工作分解为可进入 issue 的工作包。
- 为每个工作包定义 owner、依赖、验收标准、风险。
- 将 `v0.2.0` 的发布门禁与 `M1` 验收标准绑定。

## 4. In Scope / Out of Scope

### 4.1 In Scope

- 任务生命周期建模
- 审批生命周期建模
- connector / audit 抽象
- 只读 mock 集成
- 规划接口向生命周期接口演进
- 测试策略和验证基线
- 试运行候选 workflow 选择与边界定义

### 4.2 Out of Scope

- 实际生产设备控制
- 真实 ERP / MES / CMMS 生产环境写入
- LLM provider 深度接入
- 多工厂统一主数据
- 权限系统全量实现

## 5. 成功标准

`M1` 完成的判定标准如下：

1. 系统能够表示任务从 `draft / planned / awaiting_approval / approved / executing / completed / failed` 的主状态流。
2. 系统能够表示审批从 `pending / approved / rejected / expired` 的主状态流。
3. 规划接口输出不只是静态计划，还能生成可追踪的任务聚合根或任务记录。
4. 至少具备一个 `MES` mock connector 和一个 `CMMS` mock connector，且通过统一 trait 暴露只读能力。
5. 审计接口已定义，关键状态变更可以写入 mock / in-memory audit sink。
6. 请求具备 correlation id，并在 API 与核心层之间贯通。
7. 已定义 1 条 pilot workflow，并明确试运行边界、SOP 影响、审批角色、回退策略。
8. `v0.2.0` 发布时，代码、测试、文档、changelog、tag 形成闭环。

## 6. 交付策略

### 6.1 交付原则

- 先建抽象和状态机，再接 mock connector，再扩 API。
- 所有高风险点先做设计，不直接跳进编码。
- 优先形成一条端到端最小闭环，而不是多个半成品模块。

### 6.2 工作流主线

本阶段采用 5 条并行工作流：

1. `WS1` Orchestration lifecycle
2. `WS2` Integration abstraction
3. `WS3` Audit and observability baseline
4. `WS4` Pilot workflow definition
5. `WS5` QA and release readiness

## 7. WBS

### 7.1 Epic 视图

| Epic ID | 名称 | 目标 | Owner Role | 目标完成日 |
| --- | --- | --- | --- | --- |
| M1-E1 | Task lifecycle foundation | 建立任务生命周期模型和状态迁移规则 | Architect / Eng Lead | 2026-03-25 |
| M1-E2 | Approval lifecycle foundation | 建立审批聚合和状态迁移规则 | Architect / Eng Lead | 2026-03-26 |
| M1-E3 | Connector abstraction and mocks | 定义 connector trait 并接入 MES / CMMS mock | Eng Lead | 2026-03-28 |
| M1-E4 | Audit and correlation baseline | 定义 audit trait 和 correlation id 贯通 | Eng Lead | 2026-03-28 |
| M1-E5 | API evolution for v0.2.0 | 增加生命周期相关 API 和序列化模型 | Eng Lead | 2026-03-31 |
| M1-E6 | Pilot workflow selection and readiness | 选择并定义 1 条 pilot workflow | PO / Delivery Lead / Pilot Owner | 2026-04-03 |
| M1-E7 | Test and release readiness | 完成验证、缺陷修复和 release 准备 | QA Lead / Eng Lead | 2026-04-10 |

### 7.2 详细工作包

| WBS ID | 工作包 | 主要输出 | Owner Role | 依赖 | 目标日期 | 验收标准 |
| --- | --- | --- | --- | --- | --- | --- |
| M1-W01 | 设计任务状态模型 | `TaskStatus`, 状态迁移规则, 领域文档 | Architect | 无 | 2026-03-14 | 状态与约束被文档化，代码模型可编译 |
| M1-W02 | 设计审批状态模型 | `ApprovalStatus`, 审批约束, 角色映射 | Architect | 无 | 2026-03-14 | 状态与审批角色关系清晰，可进入实现 |
| M1-W03 | 设计 connector trait | `Connector`, `ConnectorKind`, read-only contract | Architect / Integration Engineer | 无 | 2026-03-17 | trait 边界清晰，不绑定供应商 |
| M1-W04 | 设计 audit trait | `AuditSink`, `AuditEvent`, 最小事件模型 | Architect | 无 | 2026-03-17 | 能覆盖任务和审批关键事件 |
| M1-W05 | 创建 M1 issue map | issue draft list, project board mapping | Delivery Lead | M1-W01..W04 | 2026-03-18 | 所有工作包具备 issue 标题和分类 |
| M1-W06 | 实现任务聚合根 | 任务实体、状态转换方法、测试 | Engineer | M1-W01 | 2026-03-24 | 单测覆盖主状态流与非法转换 |
| M1-W07 | 实现审批聚合根 | 审批实体、状态转换方法、测试 | Engineer | M1-W02 | 2026-03-24 | 单测覆盖主审批流与非法转换 |
| M1-W08 | 将 planner 接入任务聚合 | 规划结果生成任务记录或任务草稿 | Engineer | M1-W06, M1-W07 | 2026-03-25 | `/tasks/plan` 输出可追踪任务对象 |
| M1-W09 | 实现 connector trait 与 mock MES | mock MES read API | Engineer | M1-W03 | 2026-03-26 | 统一 trait 下可读任务上下文或设备数据 |
| M1-W10 | 实现 mock CMMS connector | mock CMMS read API | Engineer | M1-W03 | 2026-03-26 | 统一 trait 下可读工单或保养记录 |
| M1-W11 | 实现 audit sink | in-memory audit sink + event writer | Engineer | M1-W04 | 2026-03-26 | 关键事件被记录并可测试 |
| M1-W12 | 引入 correlation id | API 层生成 / 透传 correlation id | Engineer | M1-W08 | 2026-03-27 | 请求与审计事件能够关联 |
| M1-W13 | 扩展 API 端点 | create / get / approve / execute stub endpoints | Eng Lead | M1-W06..W12 | 2026-03-31 | API 可演示任务生命周期主路径 |
| M1-W14 | 编写集成测试 | API、状态机、mock connector 测试 | QA Lead / Engineer | M1-W13 | 2026-03-31 | 集成测试通过并覆盖主路径 |
| M1-W15 | 定义 pilot workflow 候选集 | 3 条候选 workflow, 风险比较 | PO / Pilot Owner | 无 | 2026-03-21 | 候选流具备价值、风险、依赖分析 |
| M1-W16 | 选定 pilot workflow | 1 条被批准的 pilot workflow | PO / Sponsor / Pilot Owner | M1-W15 | 2026-03-24 | 有明确批准结论 |
| M1-W17 | 编写 pilot workflow spec | 流程图、SOP 影响、审批角色、回退方案 | Delivery Lead / Architect | M1-W16 | 2026-04-03 | 文档可作为 M2 的实现输入 |
| M1-W18 | 编写 v0.2.0 测试与发布清单 | test checklist, release checklist | QA Lead / Delivery Lead | M1-W14 | 2026-04-02 | 发布前检查项完整 |
| M1-W19 | 缺陷清理与 M1 closeout | 缺陷清理、复盘、closeout note | Eng Lead / QA Lead / Delivery Lead | M1-W18 | 2026-04-10 | M1 验收结论明确 |

## 8. 立即两周执行窗口

### 8.1 Window A: 2026-03-11 至 2026-03-20

目标：

- 完成所有关键设计结论
- 建立 issue map
- 锁定 pilot workflow 候选集
- 为 `2026-03-23` 的实现周做好准备

必须完成：

- M1-W01
- M1-W02
- M1-W03
- M1-W04
- M1-W05
- M1-W15

### 8.2 Window B: 2026-03-23 至 2026-03-31

目标：

- 交付 `v0.2.0`
- 让核心生命周期、connector、audit、correlation id 跑通

必须完成：

- M1-W06
- M1-W07
- M1-W08
- M1-W09
- M1-W10
- M1-W11
- M1-W12
- M1-W13
- M1-W14

### 8.3 Window C: 2026-04-01 至 2026-04-10

目标：

- 完成 pilot workflow 规格化
- 清理缺陷
- 形成 M1 closeout

必须完成：

- M1-W16
- M1-W17
- M1-W18
- M1-W19

## 9. Pilot workflow 选择建议

### 9.1 推荐 workflow

建议将第一条 pilot workflow 设为：

`设备温度漂移异常诊断与维修工单建议`

### 9.2 选择理由

- 与当前已有的示例任务和领域模型一致。
- 业务价值明确，制造企业普遍存在类似设备异常。
- 可以覆盖 `任务规划 -> 证据读取 -> 审批建议 -> CMMS 工单建议` 的编排链。
- 可以通过只读 MES / CMMS mock 先验证逻辑，不必一开始接真实写链路。
- 相比直接调设备参数或质量放行，这条流程风险更可控。

### 9.3 Pilot 边界

允许：

- 读取设备上下文和维护记录
- 生成诊断建议
- 生成待审批维修建议或工单草稿

不允许：

- 直接修改设备参数
- 直接下发控制指令
- 直接闭环执行真实工单写入

### 9.4 Pilot 验收要点

- 参与角色明确
- 审批点明确
- 证据来源明确
- 回退到人工流程的路径明确
- SOP 影响说明明确

## 10. 依赖与 RAID

### 10.1 关键依赖

| ID | 依赖项 | 当前状态 | Owner Role | 应对 |
| --- | --- | --- | --- | --- |
| D1 | connector trait 设计冻结 | 待完成 | Architect | 在 2026-03-17 前冻结最小接口 |
| D2 | pilot workflow 候选评估 | 待完成 | PO / Pilot Owner | 在 2026-03-21 前形成候选比较 |
| D3 | API 生命周期边界 | 待完成 | Eng Lead | 先以 stub endpoint 落地，不等待全量流程 |

### 10.2 Top risks

| Risk ID | 风险 | 影响 | 应对 |
| --- | --- | --- | --- |
| R-M1-1 | 状态机设计过度复杂 | 拖慢 v0.2.0 | 先交付最小主路径，不做过早泛化 |
| R-M1-2 | connector 抽象绑定具体系统 | 后续扩展困难 | trait 只暴露通用 read contract |
| R-M1-3 | pilot workflow 选得太大 | M2 失控 | 坚持只选低风险诊断类流程 |
| R-M1-4 | 测试策略滞后于实现 | 发布质量下降 | QA 提前介入 W14 和 checklist |
| R-M1-5 | 文档与代码脱节 | 管理基线失效 | 每个 Epic 关闭前检查文档更新 |

## 11. 验收与阶段门

### 11.1 `v0.2.0` Release Gate

进入 `Ready for Release` 前必须满足：

1. `M1-W06` 到 `M1-W14` 完成。
2. `cargo fmt --all` 通过。
3. `cargo clippy --workspace --all-targets -- -D warnings` 通过。
4. `cargo test --workspace` 通过。
5. 新 API 的手工验证证据已记录。
6. `CHANGELOG.md` 更新。

### 11.2 `M1` Closeout Gate

进入 `Closed` 前必须满足：

1. `v0.2.0` 已发布。
2. `M1-W16` 到 `M1-W19` 完成。
3. pilot workflow spec 已冻结。
4. M2 的输入材料已明确。

## 12. 建议的 GitHub issue 列表

以下标题建议直接转成 issue：

- `[M1] Define task lifecycle domain model`
- `[M1] Define approval lifecycle domain model`
- `[M1] Define connector trait and connector kinds`
- `[M1] Define audit event model and audit sink trait`
- `[M1] Implement task aggregate and transition rules`
- `[M1] Implement approval aggregate and transition rules`
- `[M1] Evolve planner to emit tracked task records`
- `[M1] Add mock MES connector`
- `[M1] Add mock CMMS connector`
- `[M1] Add in-memory audit sink`
- `[M1] Thread correlation id through API and core`
- `[M1] Add lifecycle API endpoints`
- `[M1] Add integration tests for orchestration lifecycle`
- `[M1] Evaluate pilot workflow candidates`
- `[M1] Write pilot workflow specification`
- `[M1] Prepare v0.2.0 release checklist`

## 13. 下一步执行动作

如果按此计划继续，下一轮编码和管理动作应严格按以下顺序：

1. 完成 `M1-W01` 到 `M1-W04` 的设计文档和代码骨架。
2. 把第 12 节的 issue 草案真正建到 GitHub。
3. 进入 `M1-W06` 到 `M1-W12` 的实现。
4. 在 `2026-03-31` 前完成 `v0.2.0` 并发布。

这意味着下一步最合适的实际编码任务，是先实现任务状态机、审批状态机以及 connector / audit trait。

当前设计基线文档：

- [design/m1-lifecycle-and-abstractions.md](/Users/admin/Documents/WORK/ai/fa/docs/design/m1-lifecycle-and-abstractions.md)
