# FlowBuilder

> 🚀 基于 Rust 的高性能异步工作流引擎：YAML 驱动 + 分层架构 + 可组合特性

[![Crates.io](https://img.shields.io/crates/v/flowbuilder.svg)](https://crates.io/crates/flowbuilder)
[![Documentation](https://docs.rs/flowbuilder/badge.svg)](https://docs.rs/flowbuilder)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## ✨ 特性概览

-   分层架构：解析（Parser）→ 编排（Orchestrator）→ 执行（Executor）
-   并行执行：自动推断阶段并行度，信号量限流
-   条件 / 复杂度分析：执行前预估复杂度与最大并行度
-   重试 + 超时：节点级配置
-   性能指标：任务/阶段耗时与聚合统计（可选）
-   详细日志：调试期开启（可选）
-   YAML 驱动：声明式工作流 + 验证 + 预览计划
-   纯构建 DSL：无 YAML 场景可最小化依赖

## 🚀 快速开始

### 安装 (默认已启用 core + yaml-runtime)

```toml
[dependencies]
flowbuilder = { version = "0.1.1" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### YAML 配置示例

````yaml
workflow:
    version: "1.0"
    env:
        ENVIRONMENT: "production"
        LOG_LEVEL: "info"

## 🔌 特性指南

Feature 体系（可组合）：

| 类别 | 名称 | 说明 |
| ---- | ---- | ---- |
| 基础 | core | 构建器 + 上下文 DSL |
| 解析 | yaml | YAML/JSON 解析与验证 |
| 执行 | runtime | 编排 + 高级执行（默认含 parallel/retry/perf-metrics） |
| 组合 | yaml-runtime | = yaml + runtime + 启用 yaml 中的 runtime 子特性 |
| 子特性 | parallel | 阶段/节点并行 |
| 子特性 | retry | 节点重试策略 |
| 子特性 | perf-metrics | 执行统计/耗时（runtime 默认启用） |
| 子特性 | detailed-logging | 详细阶段/节点调试日志 |

默认启用：`core + yaml-runtime`。

### 1. 最小核心

```toml
[dependencies]
flowbuilder = { version = "0.1.1", default-features = false, features = ["core"] }
tokio = { version = "1", features = ["rt","macros"] }
````

可用：`FlowBuilder` / `SharedContext`；不可用：动态 YAML / 增强执行器 / 复杂度分析 / 性能统计。

### 2. 最小 Runtime（无 YAML）

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["runtime"] }
```

包含 runtime 默认子特性：并行 + 重试 + 性能指标。

### 3. 仅 YAML 解析

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["yaml"] }
```

仅使用配置解析与验证，可用于离线分析或静态检查。

### 4. YAML + 执行（推荐）

```toml
# 方式A：显式列出
flowbuilder = { version = "0.1.1", features = ["yaml","runtime"] }
# 方式B：使用组合特性（简化推荐）
flowbuilder = { version = "0.1.1", features = ["yaml-runtime"] }
```

等价于默认启用（当前默认特性为 `core,yaml-runtime`）。

组合特性优势：

-   保证同时开启 facade 对 YAML + Runtime 的依赖链
-   避免忘记启用 `flowbuilder-yaml` 的 runtime 子特性导致执行方法缺失
-   减少依赖声明长度，提升可读性

### 5. 性能统计 (perf-metrics)

`runtime` 默认已经包含 `perf-metrics`，只需在代码中在启用特性时读取统计：

```rust
#[cfg(all(feature = "runtime", feature = "perf-metrics"))]
{
    let stats = executor.get_stats();
    println!("total={} success={} avg={:?}", stats.total_tasks, stats.successful_tasks, stats.average_execution_time);
}
```

### 6. 详细调试日志 (detailed-logging)

```toml
flowbuilder = { version = "0.1.1", features = ["runtime","yaml","detailed-logging"] }
```

该特性会在执行过程中输出更细粒度的阶段/节点日志，只建议调试时使用。

### 7. 组合速览

| 场景       | features                         | 说明                             |
| ---------- | -------------------------------- | -------------------------------- |
| 仅构建     | core                             | DSL 构建，无编排/并行            |
| 最小执行   | runtime                          | 并行 + 重试 + 指标（默认子特性） |
| 仅解析     | yaml                             | 解析 / 验证 / 计划预估（无执行） |
| YAML 执行  | yaml-runtime                     | 解析 + 编排 + 执行               |
| 调试       | yaml-runtime,detailed-logging    | 加详细日志                       |
| 指标可观测 | yaml-runtime (已含 perf-metrics) | 执行统计                         |

### 8. 规划

-   `runtime-base`：拆分默认子特性
-   分布式/分片：`distributed` / `sharding`
-   更细粒度指标导出

### 9. 快速代码示例

```rust
use flowbuilder::prelude::*;            // facade 导入
use flowbuilder::yaml::prelude::*;      // 动态执行器
#[cfg(feature = "runtime")] use flowbuilder::runtime::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let yaml = std::fs::read_to_string("workflow.yaml")?;
    let config = WorkflowLoader::from_yaml_str(&yaml)?;
    let mut exec = DynamicFlowExecutor::new(config)?; // 需要 feature = "yaml" (执行阶段需要 runtime)

    let ctx = std::sync::Arc::new(tokio::sync::Mutex::new(flowbuilder_context::FlowContext::default()));
    #[cfg(feature = "runtime")] {
        let result = exec.execute(ctx).await?;
        println!("success={} phases={}", result.success, result.phase_results.len());
        #[cfg(feature = "perf-metrics")] {
            let stats = exec.get_stats();
            println!("tasks={} ok={}", stats.total_tasks, stats.successful_tasks);
        }
    }
    Ok(())
}
```

> 提示：生产环境建议关闭 `detailed-logging`；若需最小二进制请选择 `default-features = false`。

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

````

### 更多示例

查看 `examples/new_architecture_demo.rs` 或 `docs/quick-start-guide.md`。

## 🏗️ 架构设计

## 🏗️ 架构

分层确保职责清晰 & 易扩展：

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

核心组件：配置解析器 / 流程编排器 / 任务执行器 / 上下文状态管理

## 📊 性能

零成本抽象 / 异步优先 / 并行调度 / 指标可观测 / 内存友好

## 🔧 配置驱动

### 支持的配置能力

任务定义 / 依赖管理 / 条件执行 / 重试 / 超时 / 变量注入
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

## 📚 文档

-   [快速入门](docs/quick-start-guide.md) - 安装与基本使用
-   [架构设计](docs/architecture.md) - 分层架构与核心组件
-   [API 参考](docs/api-reference.md) - 公共接口与特性说明（同步至 docs.rs）

## 📝 示例

查看 `examples/new_architecture_demo.rs` 获取完整的使用示例。

## 🧪 测试 & 示例

```bash
cargo test
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

## 🔌 Import 与最小化

示例（自定义精简）：

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["core", "runtime", "parallel"] }
```

#### 仅核心

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core"] }
```

```rust
use flowbuilder::prelude::*; // 仅包含 FlowBuilder / 基础执行抽象
```

#### 核心 + 执行（无 YAML）

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core", "runtime", "parallel", "retry"] }
```

```rust
use flowbuilder::runtime::prelude::*; // EnhancedTaskExecutor / EnhancedFlowOrchestrator 等
```

关闭性能统计与详细日志：

```toml
features = ["core", "runtime", "parallel", "retry"]  # 不包含 perf-metrics / detailed-logging
```

#### YAML 动态执行

```toml
[dependencies]
flowbuilder = { version = "0.1.0", features = ["yaml", "runtime"] } # 默认还包含 core
```

```rust
use flowbuilder::yaml::prelude::*;      // 动态加载器 / DynamicFlowExecutor
use flowbuilder::runtime::prelude::*;   // 高级执行器 (需要 runtime)
```

#### 极致精简

```toml
flowbuilder = { version = "0.1.0", default-features = false, features = ["core"] }
```

#### 全特性

```toml
flowbuilder = { version = "0.1.0", features = ["yaml", "runtime", "parallel", "retry", "perf-metrics", "detailed-logging"] }
```

### Import 建议

| 需求场景        | 推荐 imports                                                | 说明                  |
| --------------- | ----------------------------------------------------------- | --------------------- |
| 纯构建/串行流程 | `use flowbuilder::prelude::*;`                              | 不启用 runtime 子特性 |
| 高性能执行      | `use flowbuilder::runtime::prelude::*;`                     | 并行 / 重试 / 指标    |
| YAML 驱动       | `use flowbuilder::yaml::prelude::*;` + 视需要再引入 runtime | 分离解析与执行        |
| 最小二进制      | 仅 core                                                     | 减少依赖与代码大小    |

> FAQ: 为什么不在 YAML prelude 里自动带 runtime? —— 避免无意拉入并行/重试/指标等额外逻辑，给予使用者显式控制权。
