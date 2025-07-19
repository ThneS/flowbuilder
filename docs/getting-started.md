# FlowBuilder 快速入门指南

FlowBuilder 是一个基于 Rust 的企业级异步工作流引擎，采用分层架构设计，支持 YAML 配置驱动的工作流执行。

## 🚀 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["yaml", "runtime"] }
```

### 基本使用

```rust
use flowbuilder::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flow = FlowBuilder::new()
        .step(|_ctx| async move {
            println!("执行步骤 1");
            Ok(())
        })
        .step(|_ctx| async move {
            println!("执行步骤 2");
            Ok(())
        })
        .build();

    flow.execute().await?;
    Ok(())
}
```

## 🏗️ 架构设计

FlowBuilder 采用分层架构：

```
┌─────────────────────┐
│   YAML 配置文件     │
└─────────────────────┘
           ↓
┌─────────────────────┐
│  YamlConfigParser   │  ← 配置解析器
└─────────────────────┘
           ↓
┌─────────────────────┐
│ EnhancedOrchestrator│  ← 流程编排器
└─────────────────────┘
           ↓
┌─────────────────────┐
│  EnhancedExecutor   │  ← 任务执行器
└─────────────────────┘
```

### 核心组件

-   **配置解析器**: 解析 YAML 配置，生成执行节点
-   **流程编排器**: 创建执行计划，优化执行顺序
-   **任务执行器**: 执行具体的任务，支持并行、重试、超时

## 📝 YAML 配置示例

```yaml
workflow:
    version: "1.0"
    env:
        ENVIRONMENT: "production"
        LOG_LEVEL: "info"
    vars:
        max_retries: 3
        timeout: 30
    tasks:
        - task:
              id: "setup"
              name: "环境设置"
              description: "初始化执行环境"
              actions:
                  - action:
                        id: "init"
                        name: "初始化"
                        type: "builtin"
                        flow:
                            retry:
                                max_retries: 2
                                delay: 1000
                            timeout:
                                duration: 5000
        - task:
              id: "process"
              name: "数据处理"
              description: "处理业务数据"
              actions:
                  - action:
                        id: "process_data"
                        name: "数据处理"
                        type: "builtin"
```

## 🔧 功能特性

### 核心特性

-   ✅ **YAML 配置驱动**: 支持完整的 YAML 工作流配置
-   ✅ **分层架构**: 清晰的配置解析 → 编排 → 执行分层
-   ✅ **异步执行**: 原生 async/await 支持
-   ✅ **并行执行**: 支持任务并行执行
-   ✅ **错误处理**: 完善的错误处理和恢复机制
-   ✅ **重试机制**: 可配置的重试策略
-   ✅ **超时控制**: 任务级和全局超时控制

### 高级特性

-   ✅ **执行计划**: 自动生成优化的执行计划
-   ✅ **复杂度分析**: 工作流复杂度分析和优化建议
-   ✅ **上下文管理**: 线程安全的上下文状态管理
-   ✅ **可观测性**: 详细的执行日志和指标

## 📚 使用示例

查看 `examples/new_architecture_demo.rs` 获取完整的使用示例。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

Apache License 2.0
