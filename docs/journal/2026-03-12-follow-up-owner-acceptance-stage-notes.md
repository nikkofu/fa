# Follow-up Owner Acceptance Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

只有 seeded follow-up draft 还不够，因为这只能说明系统看见了待办，也给出了推荐角色，不能说明系统真的知道“谁接下了这条待办”。

这一层单独拿出来，就是为了让 `accepted owner` 第一次拥有显式 action。

## 2. 这一阶段的创新点

这一阶段的关键，是没有一上来做完整 assignment 系统、queue 或 item aggregate，而是先加一条最小显式 action：

- 只对任务详情里已存在的 `follow_up_item` 生效
- 只允许 `draft -> accepted`
- 只允许匹配 `recommended_owner_role` 的 actor 执行

这意味着平台开始明确：

- recommended owner 与 accepted owner 是两种不同语义
- owner acceptance 不需要等 cross-task queue 才能成立
- handoff receipt 和 item-level acceptance 可以在同一 task detail 主链里并列存在但不混淆

## 3. 这如何改变世界

制造现场很多协同失败，不是因为系统没写出下一步，而是因为系统根本不能表达“谁已经真正接住了下一步”。

只要没有显式 acceptance action，平台就更像“会生成建议列表的系统”，而不是“会推动待办接手闭环的系统”。

## 4. 对自己的要求

- 不把 accepted owner 夸大成任务已经完成
- 不把 handoff receipt acknowledgement 误写成 item owner acceptance
- 不为了这一步引入过重的新状态机、projection 或 queue

## 5. 已经验证的事实

- `shift handoff` 与 `alert triage` 的 seeded `follow_up_item` 现在能通过显式 action 从 `draft` 进入 `accepted`
- 不匹配 `recommended_owner_role` 的 actor 会被拒绝
- 已接受的 item 会被拒绝重复 acceptance
- sandbox-safe file mode 重启后仍可回读 accepted follow-up 与对应 `follow_up_owner_accepted` audit event
- `shift handoff` receipt summary 会在 acceptance 后同步减少未接手计数

## 6. 这次做对了什么

这次做对的地方，是没有急着去做 assignment matrix、owner queue 或 overdue engine，而是继续沿 task-scoped 主链把“推荐 owner 如何变成 accepted owner”这条最小路径补齐。

这样后续不论是扩 item assignment，还是扩 cross-task owner queue，都能沿同一条低风险、可验证的路线继续推进。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让高频 follow-up 从“系统推荐谁做”进入了“系统知道谁接住了”。

这会直接影响后续路线：

- `follow_up_item` 第一次具备 `draft -> accepted` 的显式审计回放语义
- `shift handoff` receipt 与 item-level acceptance 的边界开始在代码里成立
- 后续 assignment / queue / overdue 视图可以建立在已存在的 owner acceptance 动作之上，而不是建立在猜测上
