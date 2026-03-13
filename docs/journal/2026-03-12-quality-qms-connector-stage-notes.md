# Quality QMS Connector Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

质量场景最容易产生的错觉，是代码里已经有 `Quality`、`QualityContext` 这些词，于是大家会误以为质量证据链路已经差不多通了。但“枚举里有”和“默认运行主链里能读到 evidence”完全不是一回事。

如果 `QMS` baseline 不单独做，质量场景就永远只有对齐清单，没有真实证据入口。

## 2. 这一阶段的创新点

这一阶段的关键，是把质量问题从“是否有质量概念”推进到“质量证据如何第一次进入默认运行主链”。

这意味着平台开始明确：

- 哪些记录是第一版 `QMS` 必须返回的
- 为什么第一版先复用 `QualityContext` 就够了
- `QMS` baseline 应该怎样和 `MES` 形成最小联动

## 3. 这如何改变世界

制造业里的很多质量协同损耗，并不是因为没有偏差流程，而是因为系统层很难快速把：

- 偏差主记录
- 检验结果
- 当前处置状态

稳定地拉进同一条证据链。

`mock QMS` baseline 真正改变的，就是这层“质量证据第一次可见”的问题。

## 4. 对自己的要求

- 不把 `Quality` target 写成 evidence 已经打通
- 不把 connector baseline 和审批策略混成一个问题
- 不让第一版 `QMS` mock 变成另一个过度设计的质量平台

## 5. 已经验证的事实

- 当前 `IntegrationTarget::Quality`、`ConnectorKind::Quality`、`QualityContext` 都已经存在
- 当前 `ConnectorRegistry::with_m1_defaults()` 并没有注册 `Quality`
- 当前 `requested_record_kinds()` 也不会为 `Quality` 生成有效读取计划

## 6. 这次做对了什么

这次做对的地方，是没有继续空谈质量 workflow，而是把问题拉回默认运行主链里最具体、最可验证的一步：先让 `QMS` evidence 不再为空。

这样后续无论是做 `Quality Manager` 审批策略，还是做 containment draft，都有了真正的证据起点。

## 7. 这一步如何真正产生影响

这份 baseline note 的真正价值，在于它让 `质量偏差隔离与处置建议` 从“已对齐的候选 workflow”继续推进到“可以进入演示链路的 connector 基线”。

这会直接影响后续路线：

- quality task evidence 会第一次变得可见
- audit 会开始记录 `Quality` connector read
- 质量场景会开始具备区别于纯设备异常的独立证据深度
