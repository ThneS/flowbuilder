# 安全策略

## 报告安全漏洞

我们非常重视 FlowBuilder 的安全问题。如果您发现了安全漏洞，请通过以下方式报告：

1. **不要**在 GitHub Issues 中公开报告安全问题
2. 发送邮件到 security@thnes.com
3. 或者通过 [GitHub 安全咨询](https://github.com/ThneS/flowbuilder/security/advisories/new) 私下报告

## 安全响应流程

1. 我们会在 48 小时内确认收到您的报告
2. 我们会尽快评估报告的问题
3. 如果确认是安全问题，我们会：
   - 在 7 天内发布修复版本
   - 在修复发布后公开披露问题
   - 将您添加到安全公告的致谢名单中

## 安全更新

- 我们会在 [CHANGELOG.md](../CHANGELOG.md) 中记录所有安全更新
- 安全更新会通过新的补丁版本发布
- 严重的安全问题会通过新的次要版本发布

## 受支持的版本

我们支持以下版本的安全更新：

| 版本 | 支持状态 | 安全更新截止日期 |
|------|----------|------------------|
| 1.x  | 活跃支持 | 2025-12-31       |
| 0.x  | 维护支持 | 2024-12-31       |

## 安全最佳实践

1. 始终使用最新版本的 FlowBuilder
2. 定期检查 [CHANGELOG.md](../CHANGELOG.md) 了解安全更新
3. 遵循 [文档](../docs/getting-started.md) 中的安全建议
4. 使用 `cargo audit` 检查依赖项的安全问题

## 安全特性

FlowBuilder 实现了以下安全特性：

1. 内存安全：利用 Rust 的所有权系统
2. 线程安全：使用 Rust 的并发原语
3. 错误处理：全面的错误处理机制
4. 输入验证：严格的数据验证
5. 资源管理：自动资源清理

## 安全依赖

我们：

1. 定期更新依赖项
2. 使用 `cargo audit` 检查依赖项
3. 优先使用经过审计的依赖项
4. 最小化依赖项数量

## 安全联系方式

- 安全团队邮箱：security@thnes.com
- PGP 公钥：[security.asc](../security.asc)
- 安全公告：[GitHub 安全公告](https://github.com/ThneS/flowbuilder/security/advisories)

## 致谢

感谢所有报告安全问题的贡献者。您的帮助使 FlowBuilder 更加安全。