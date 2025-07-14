# FlowBuilder

> 🚀 **企业级异步工作流引擎** - 基于 Rust 的高性能工作流引擎，支持 YAML 配置驱动、分层架构设计

[![Crates.io](https://img.shields.io/crates/v/flowbuilder.svg)](https://crates.io/crates/flowbuilder)
[![Documentation](https://docs.rs/flowbuilder/badge.svg)](https://docs.rs/flowbuilder)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## ✨ 核心特性

### �️ **分层架构设计**

-   ✅ **配置解析器** - YAML 配置解析和验证
-   ✅ **流程编排器** - 智能执行计划生成和优化
-   ✅ **任务执行器** - 高性能任务执行和控制
-   ✅ **统一接口** - 清晰的分层抽象和标准接口

### ⚡ **高性能执行**

-   ✅ **并行执行** - 自动分析依赖，最大化并行度
-   ✅ **异步原生** - 基于 Tokio 的零成本异步抽象
-   ✅ **资源控制** - 可配置的并发限制和背压控制
-   ✅ **执行优化** - 智能执行计划优化

### � **YAML 配置驱动**

-   ✅ **声明式配置** - 完整的 YAML 工作流定义
-   ✅ **配置验证** - 自动配置完整性检查
-   ✅ **环境变量** - 支持环境变量和流程变量
-   ✅ **热重载** - 支持配置动态加载

### 🛡️ **企业级可靠性**

-   ✅ **错误恢复** - 多层次错误处理和恢复机制
-   ✅ **重试策略** - 可配置的智能重试
-   ✅ **超时控制** - 任务级和全局超时管理
-   ✅ **可观测性** - 完整的执行追踪和指标

## 🚀 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["yaml", "runtime"] }
tokio = { version = "1.0", features = ["full"] }
```

### YAML 配置示例

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

### 代码示例

```rust
use flowbuilder_yaml::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 YAML 配置创建执行器
    let yaml_content = std::fs::read_to_string("workflow.yaml")?;
    let mut executor = DynamicFlowExecutor::from_yaml(&yaml_content)?;

    // 创建执行上下文
    let context = std::sync::Arc::new(tokio::sync::Mutex::new(
        flowbuilder_context::FlowContext::default()
    ));

    // 执行工作流
    let result = executor.execute(context).await?;

    println!("工作流执行完成: {}", result.success);
    println!("总耗时: {:?}", result.total_duration);
    println!("执行节点数: {}", result.nodes_executed);

    Ok(())
}
```

## 🏗️ 架构设计

FlowBuilder 采用分层架构设计，确保高性能、可扩展性和易维护性：

```
┌─────────────────────┐
│   YAML 配置文件     │
└─────────────────────┘
           ↓
┌─────────────────────┐
│  YamlConfigParser   │  ← 配置解析器
│  • 解析 YAML 配置   │
│  • 验证配置完整性   │
│  • 生成执行节点     │
└─────────────────────┘
           ↓
┌─────────────────────┐
│ EnhancedOrchestrator│  ← 流程编排器
│  • 创建执行计划     │
│  • 优化执行顺序     │
│  • 分析工作流复杂度 │
└─────────────────────┘
           ↓
┌─────────────────────┐
│  EnhancedExecutor   │  ← 任务执行器
│  • 执行具体任务     │
│  • 并行执行控制     │
│  • 重试和超时处理   │
└─────────────────────┘
```

### 核心组件

-   **配置解析器**: 负责 YAML 配置的解析、验证和结构化
-   **流程编排器**: 创建优化的执行计划，处理依赖关系
-   **任务执行器**: 高性能的任务执行，支持并行、重试、超时等

## 📊 性能特点

-   **零成本抽象** - 编译时优化，运行时高效
-   **异步优先设计** - 原生 Tokio 集成，高并发支持
-   **内存高效** - 最小化内存分配和复制
-   **智能并行** - 自动分析依赖，最大化并行执行机会

## 🔧 配置驱动

### 支持的配置特性

-   **任务定义** - 声明式任务和动作配置
-   **依赖管理** - 自动处理任务间依赖关系
-   **重试策略** - 可配置的重试次数和延迟
-   **超时控制** - 任务级和全局超时设置
-   **环境变量** - 支持环境变量和流程变量
-   **条件执行** - 基于条件的任务执行控制
    guard.set_variable("important_data".to_string(), "original".to_string());
    Ok(())
    })
    .create_snapshot("checkpoint", "Before risky operation")
    .step_with_rollback("risky_operation", "auto_checkpoint", |ctx| async move {
    let mut guard = ctx.lock().await;
    guard.set_variable("important_data".to_string(), "modified".to_string());
    // This will fail and trigger automatic rollback
    anyhow::bail!("Operation failed")
    })
    .named_step("verify", |ctx| async move {
    let guard = ctx.lock().await;
    // Data is automatically rolled back to "original"
    assert_eq!(guard.get_variable("important_data"), Some(&"original".to_string()));
    Ok(())
    })
    .run_all()
    .await?;

````

### 4. **Distributed Tracing | 分布式追踪**

```rust
// With custom trace ID
FlowBuilder::new()
    .named_step("service_a", |_ctx| async move {
        println!("Processing in service A");
        Ok(())
    })
    .named_step("service_b", |_ctx| async move {
        println!("Processing in service B");
        Ok(())
    })
    .run_all_with_trace_id("user-request-12345".to_string())
    .await?;

// Output includes trace ID in all logs:
// [trace_id:user-request-12345] [step:service_a] starting...
// [trace_id:user-request-12345] [step:service_a] completed successfully in 1.2ms
````

### 5. **Error Handling Strategies | 错误处理策略**

```rust
FlowBuilder::new()
    // Continue on error (don't stop the flow)
    .step_continue_on_error("optional_step", |_ctx| async move {
        anyhow::bail!("This error won't stop the flow")
    })
    // Handle errors with custom logic
    .step_handle_error("critical_step",
        |_ctx| async move {
            anyhow::bail!("Critical error")
        },
        |ctx, error| {
            ctx.set_variable("error_handled".to_string(), "true".to_string());
            println!("Handled error: {}", error);
            Ok(())
        }
    )
    // Wait until condition is met
    .step_wait_until("wait_for_recovery",
        |ctx| ctx.get_variable("error_handled").is_some(),
        Duration::from_millis(100),
        10
    )
    .run_all()
    .await?;
```

## 🏗️ Architecture | 架构设计

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   FlowBuilder   │───▶│   SharedContext  │───▶│   StepResults   │
│                 │    │                  │    │                 │
│ • Step Chain    │    │ • Variables      │    │ • Trace Logs    │
│ • Parallel Exec │    │ • Snapshots      │    │ • Performance   │
│ • Error Handle  │    │ • Error State    │    │ • Error Details │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Core Components | 核心组件

-   **FlowBuilder**: Workflow definition and execution engine | 工作流定义与执行引擎
-   **SharedContext**: Thread-safe state management with snapshots | 线程安全的状态管理
-   **StepLog**: Comprehensive execution tracking and metrics | 完整的执行追踪与指标
-   **Parallel Engine**: High-performance concurrent step execution | 高性能并发步骤执行

## 📊 Performance | 性能特点

-   **Zero-cost abstractions** - Compile-time optimizations | 零成本抽象
-   **Async-first design** - Native tokio integration | 异步优先设计
-   **Memory efficient** - Minimal allocation overhead | 内存高效
-   **Scale to thousands** - Concurrent flows and steps | 支持千级并发

## 🔧 Configuration | 配置选项

### Timeout Settings | 超时设置

```rust
// Step-level timeout
.step_with_timeout("api_call", Duration::from_secs(30), handler)

// Flow-level timeout
.run_all_with_timeout(Duration::from_minutes(5))
```

### Retry Strategies | 重试策略

```rust
.step_with_retry("flaky_operation", 3, Duration::from_secs(1), handler)
```

### Parallel Configuration | 并行配置

```rust
.parallel_steps_with_join("batch_process", subflows)  // Wait for all
.parallel_steps(subflows)  // Fire and forget
```

## 🧪 Testing | 测试

Run all tests:

```bash
cargo test
```

Run specific test suites:

## 📚 文档

-   [快速入门](docs/getting-started.md) - 安装和基本使用
-   [架构设计](docs/architecture.md) - 分层架构详解
-   [API 参考](docs/api-reference.md) - 完整 API 文档

## 📝 示例

查看 `examples/new_architecture_demo.rs` 获取完整的使用示例。

## 🧪 测试

运行所有测试：

```bash
cargo test
```

运行示例：

```bash
cargo run --example new_architecture_demo
```

## 🌟 使用场景

-   **微服务编排** - 微服务间的复杂工作流协调
-   **数据管道** - ETL 数据处理流程
-   **CI/CD 自动化** - 构建和部署工作流
-   **业务流程自动化** - 企业业务流程数字化
-   **API 工作流** - RESTful API 调用链编排
-   **批处理作业** - 大规模数据批处理任务

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解贡献指南。

## 📄 许可证

本项目采用 Apache License 2.0 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

用 ❤️ 为 Rust 社区打造
