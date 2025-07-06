# FlowBuilder

> 🚀 **Enterprise-grade async workflow engine for Rust** - 支持分布式追踪、超时控制、并行执行、快照回滚的生产级异步工作流引擎

[![Crates.io](https://img.shields.io/crates/v/flowbuilder.svg)](https://crates.io/crates/flowbuilder)
[![Documentation](https://docs.rs/flowbuilder/badge.svg)](https://docs.rs/flowbuilder)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## ✨ Features | 核心特性

### 🎯 **Flow Control | 流程控制**

-   ✅ **Conditional branching** - `step_if()`, `subflow_if()` | 条件分支
-   ✅ **Loops & waits** - `step_while()`, `step_wait_until()` | 循环等待
-   ✅ **Nested subflows** - Complex workflow composition | 嵌套子流程
-   ✅ **Error handling** - Continue on error or auto-rollback | 错误处理

### ⚡ **Performance & Concurrency | 性能并发**

-   ✅ **Parallel execution** - `parallel_steps_with_join()` | 并行执行
-   ✅ **Timeout control** - Step-level & flow-level timeouts | 超时控制
-   ✅ **Async/await native** - Zero-cost abstractions | 原生异步支持
-   ✅ **Resource management** - Automatic cleanup | 资源管理

### 🔍 **Observability | 可观测性**

-   ✅ **Distributed tracing** - Unique trace IDs across flows | 分布式追踪
-   ✅ **Performance metrics** - Step timing & execution stats | 性能指标
-   ✅ **Structured logging** - Rich context information | 结构化日志
-   ✅ **Error tracking** - Detailed error propagation | 错误追踪

### 🛡️ **Reliability | 可靠性**

-   ✅ **State snapshots** - Create checkpoints for rollback | 状态快照
-   ✅ **Auto-rollback** - Automatic recovery on failure | 自动回滚
-   ✅ **Retry mechanisms** - Configurable retry strategies | 重试机制
-   ✅ **Circuit breakers** - Fail-fast patterns | 熔断器模式

## 🚀 Quick Start | 快速开始

Add to your `Cargo.toml`:

```toml
[dependencies]
flowbuilder = "0.0.2"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Example | 基础示例

```rust
use flowbuilder::prelude::*;
use anyhow::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "0".to_string());
            println!("Setup completed");
            Ok(())
        })
        .step_if(
            |ctx| ctx.get_variable("counter").is_some(),
            |ctx| async move {
                println!("Counter exists, proceeding...");
                Ok(())
            }
        )
        .named_step("finish", |_ctx| async move {
            println!("Flow completed!");
            Ok(())
        })
        .run_all()
        .await?;

    Ok(())
}
```

## 📖 Advanced Examples | 高级示例

### 1. **Timeout Control | 超时控制**

```rust
FlowBuilder::new()
    .step_with_timeout("api_call", Duration::from_secs(5), |_ctx| async move {
        // Your long-running operation
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("API call completed within timeout");
        Ok(())
    })
    .run_all_with_timeout(Duration::from_secs(30)) // Overall flow timeout
    .await?;
```

### 2. **Parallel Execution with Join | 并行执行与聚合**

```rust
FlowBuilder::new()
    .parallel_steps_with_join("health_checks", vec![
        || FlowBuilder::new()
            .named_step("database_check", |_ctx| async move {
                // Database health check
                Ok(())
            }),
        || FlowBuilder::new()
            .named_step("api_check", |_ctx| async move {
                // API health check
                Ok(())
            }),
        || FlowBuilder::new()
            .named_step("cache_check", |_ctx| async move {
                // Cache health check
                Ok(())
            }),
    ])
    .named_step("verify_results", |ctx| async move {
        let guard = ctx.lock().await;
        let success = guard.get_variable("health_checks_parallel_success").unwrap();
        println!("Health checks passed: {}", success);
        Ok(())
    })
    .run_all()
    .await?;
```

### 3. **State Snapshots & Rollback | 状态快照与回滚**

```rust
FlowBuilder::new()
    .named_step("setup", |ctx| async move {
        let mut guard = ctx.lock().await;
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
```

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
```

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

```bash
cargo test --test trace_tests          # Tracing functionality
cargo test --test advanced_features    # Advanced features
cargo test --test flow_test            # Basic flow tests
```

## 📚 Documentation | 文档

-   [API Reference](docs/api-reference.md) - Complete API documentation | 完整 API 文档
-   [Advanced Usage](docs/advanced-usage.md) - Complex patterns and best practices | 高级用法和最佳实践
-   [Getting Started](docs/getting-started.md) - Tutorial and examples | 教程和示例
-   [Trace Features](docs/trace-features.md) - Observability and debugging | 可观测性和调试

## 🤝 Contributing | 贡献

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License | 许可证

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🌟 Use Cases | 使用场景

-   **Microservice orchestration** | 微服务编排
-   **Data pipeline processing** | 数据管道处理
-   **CI/CD workflow automation** | CI/CD 工作流自动化
-   **Distributed system coordination** | 分布式系统协调
-   **Business process automation** | 业务流程自动化
-   **ETL data transformation** | ETL 数据转换

---

Made with ❤️ for the Rust community | 为 Rust 社区用 ❤️ 制作
