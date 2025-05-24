# FlowBuilder 入门指南

## 简介

FlowBuilder 是一个灵活的异步 Rust 流程引擎，它允许你以声明式的方式构建复杂的异步工作流。通过 FlowBuilder，你可以轻松地：

- 链式执行异步步骤
- 在步骤间共享上下文
- 实现条件执行
- 处理重试和等待逻辑
- 优雅地处理错误
- 创建嵌套子流程

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
flowbuilder = "0.1.0"
```

## 快速开始（已移除）：

示例文件（如 basic_usage.rs、subflow_example.rs、parallel_execution.rs）已移除，请参考项目文档或后续更新。

## 最佳实践

1. **命名步骤**
   - 始终为重要步骤提供有意义的名称
   - 使用清晰的命名约定

2. **错误处理**
   - 为每个可能失败的步骤提供错误处理
   - 在上下文中记录错误信息
   - 实现适当的恢复策略

3. **上下文管理**
   - 使用类型安全的方法访问上下文数据
   - 避免在上下文中存储过大的数据
   - 及时清理不再需要的数据

4. **条件执行**
   - 使用清晰的条件表达式
   - 避免复杂的嵌套条件
   - 考虑使用子流程处理复杂条件

## 下一步

- 查看 [高级用法](advanced-usage.md) 了解更多特性
- 阅读 [API 参考](api-reference.md) 获取详细文档
- 查看 [示例文件](../examples/) 获取更多使用示例

示例文件（已移除）：
- 示例文件（如 basic_usage.rs、subflow_example.rs、parallel_execution.rs）已移除，请参考项目文档或后续更新。