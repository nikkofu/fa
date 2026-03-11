# Release Process

## 1. 版本策略

项目采用 Semantic Versioning：

- `MAJOR`: 不兼容接口或架构变更
- `MINOR`: 向后兼容的新能力
- `PATCH`: 缺陷修复和非破坏性改进

## 2. 分支与发布约束

- `main` 保持可构建
- 所有版本说明同步到 `CHANGELOG.md`
- GitHub tag 使用 `vX.Y.Z`

## 3. 发布前检查

发布前必须完成：

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. 更新 `CHANGELOG.md`
5. 确认试运行或目标环境的回退方案

## 4. 发布动作

1. 在 `main` 合并完成后更新版本号
2. 提交 changelog
3. 打 tag，例如 `v0.1.0`
4. push tag 到 GitHub
5. 由 GitHub Actions 自动创建 release

## 5. Release note 模板

每个 release 至少包含：

- 本次新增能力
- 本次修复问题
- 影响范围
- 升级注意事项
- 已知风险

## 6. 试运行前额外要求

对涉及制造现场的版本，还必须补充：

- SOP 影响评估
- 审批责任人清单
- 回退与人工接管策略
- 数据保留和审计策略
