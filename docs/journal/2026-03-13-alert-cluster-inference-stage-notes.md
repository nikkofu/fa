# Alert Cluster Inference Stage Notes - 2026-03-13

## 1. 为什么这一层值得先做

如果 `alert_cluster_draft` 永远只会产出一类固定形状，它就更像一个 demo 样例，而不是能承接真实现场告警差异的协同对象。

现场至少有两类非常不同的高频模式：

- `andon` 式重复告警突发
- `scada` 式持续阈值越界或漂移

这两类如果还被压成同一个草稿形状，后续 owner 路由、窗口判断和升级边界都会越来越假。

## 2. 这一阶段的关键做法

这一步仍然坚持最窄实现：

- 不新增 `alert cluster` queue
- 不新增 ingestion API
- 不新增 `Scada` mock connector

只增强现有 task-scoped draft inference：

- `source_system`
- `line_id`
- `triage_label`
- `recommended_owner_role`
- `cluster window`

## 3. 这如何改变世界

制造现场并不缺告警信号，缺的是对不同信号形态的稳定理解。

当系统开始区分：

- 重复告警应该优先交给 `production_supervisor`
- 持续阈值漂移应该更早路由给 `maintenance_engineer`

它才真正开始从“会生成一条告警草稿”变成“会理解不同告警协同路径”的系统。

## 4. 这次坚持的边界

- 不把 richer inference 夸大成事件系统落地
- 不让第二类 `scada` 形态破坏第一类 `andon` 形态
- 不在没有 queue 语义前就先造 projection

## 5. 已经验证的事实

- 现有 `andon` alert triage draft 现在可稳定输出 `line_pack_04`
- `scada` 阈值类请求现在可生成 `sustained_threshold_review`
- 第二类 draft 会推断 `source_system=scada / line_id=line_mix_02 / recommended_owner_role=maintenance_engineer`
- 第二类 draft 的 cluster window 会扩到 `15m`
- sandbox-safe file mode 重启后仍可回读 richer inference 结果

## 6. 这次做对了什么

这次做对的地方，是没有先去做更重的 queue，而是先把 task-scoped draft 的语义做真。

如果连单任务里的 cluster draft 还不能稳定区分不同现场模式，那么后续任何 cross-task queue 都只会放大错误语义。

## 7. 这一步如何继续往前推

这一步的真正意义，在于它让 `alert_cluster_drafts` 从“单一 seed 示例”推进到“可表达两类高频告警形态”的 read model。

后续路线会更清晰：

- richer draft 现在已经有继续做 cross-task `alert cluster` queue 的基础
- `Scada / Andon` mock connector 何时补齐，可以基于真实读层需求决定
- event ingestion 进入平台时，也更容易和现有 triage label / owner route 对齐
