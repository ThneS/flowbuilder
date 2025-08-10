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

flowbuilder = { version = "0.1.0", features = ["yaml", "runtime"] }
tokio = { version = "1.0", features = ["full"] }

````

### YAML 配置示例

```yaml
workflow:
    version: "1.0"
    env:
        ENVIRONMENT: "production"
        LOG_LEVEL: "info"

## 🔌 特性使用指南 (Feature Usage)

本项目通过 feature 组合实现按需裁剪（减少编译/体积）。当前层次关系：

- 顶层特性：`core` / `runtime` / `yaml`
- Runtime 子特性：`parallel` / `retry` / `perf-metrics` / `detailed-logging`
- 启用 `runtime` 时会自动拉入 runtime crate（其默认启用 `parallel` `retry` `perf-metrics`）。
- 顶层 crate 的子特性只是透传到 runtime；目前还不支持单独关闭 runtime crate 的默认子特性（后续可能提供 `runtime-base` + 显式子特性方式）。

### 1. 最小核心 (仅构建器)

```toml
[dependencies]
flowbuilder = { version = "0.1.1", default-features = false, features = ["core"] }
tokio = { version = "1", features = ["rt","macros"] }
````

可用：`FlowBuilder` / `SharedContext`；不可用：动态 YAML / 增强执行器 / 复杂度分析 / 性能统计。

### 2. 最小 Runtime (不含 YAML)

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["runtime"] }
```

包含 runtime 默认子特性：并行 + 重试 + 性能指标。

### 3. YAML 解析 (不执行计划)

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["yaml"] }
```

仅使用配置解析与验证，可用于离线分析或静态检查。

### 4. YAML + 执行 (推荐基础)

```toml
flowbuilder = { version = "0.1.1", features = ["yaml","runtime"] }
```

等价于默认启用（默认特性即 `core,yaml,runtime`）。

### 5. 启用性能统计 (perf-metrics)

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

### 7. 组合示例对照

| 场景        | 依赖声明                      | 可用能力           |
| ----------- | ----------------------------- | ------------------ |
| 仅构建      | core                          | 构建/执行链式步骤  |
| 执行(最小)  | runtime (默认子特性)          | 并行+重试+统计     |
| YAML 解析   | yaml                          | 配置解析/验证/预览 |
| YAML + 执行 | yaml,runtime                  | 解析+计划+执行     |
| 全功能调试  | yaml,runtime,detailed-logging | + 调试日志         |

### 8. 未来规划（特性粒度）

-   提供 `runtime-base`（关闭并行/重试/统计默认启用）
-   独立 `metrics` 输出结构向 facade 暴露并稳定 API
-   分布式执行拓展特性：`distributed` / `sharding` (规划中)

### 9. 简单 YAML + Runtime 代码片段

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

> 提示：关闭 `detailed-logging` 可减少 IO；当前无法单独关闭并行/重试/统计（因 runtime crate 默认特性），计划在后续版本引入更细粒度控制。

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
````

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

-   [快速入门](docs/quick-start-guide.md) - 安装与基本使用
-   [架构设计](docs/architecture.md) - 分层架构与核心组件
-   [API 参考](docs/api-reference.md) - 公共接口与特性说明（同步至 docs.rs）

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

## 🔌 特性矩阵 (Features)

| Feature          | 默认             | 说明                                      |
| ---------------- | ---------------- | ----------------------------------------- |
| core             | ✔                | 核心构建器与执行抽象                      |
| runtime          | ✔                | 高级执行器/编排器聚合                     |
| yaml             | ✔                | YAML/JSON 动态加载                        |
| parallel         | ✔ (runtime 默认) | 启用阶段/节点并行与信号量控制             |
| retry            | ✔ (runtime 默认) | 节点级重试策略 (Fixed/Exponential/Linear) |
| perf-metrics     | ✔ (runtime 默认) | 执行统计与平均耗时采集                    |
| detailed-logging | ✘                | 详细阶段/节点调试日志                     |

精简体积：关闭 `detailed-logging perf-metrics retry parallel` 以获得最小执行核心。

示例：

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core", "runtime", "parallel"] }
```

### 组合使用示例（Imports & Features）

> 注意：`yaml` crate 不再隐式 re-export runtime 类型；需要同时使用时请显式启用 `runtime`。

#### 1. 仅核心（最小体积）

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core"] }
```

```rust
use flowbuilder::prelude::*; // 仅包含 FlowBuilder / 基础执行抽象
```

#### 2. 核心 + 高级执行（无 YAML）

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

#### 3. YAML 动态加载（含 runtime）

```toml
[dependencies]
flowbuilder = { version = "0.1.0", features = ["yaml", "runtime"] } # 默认还包含 core
```

```rust
use flowbuilder::yaml::prelude::*;      // 动态加载器 / DynamicFlowExecutor
use flowbuilder::runtime::prelude::*;   // 高级执行器 (需要 runtime)
```

#### 4. 极致精简（仅构建 DSL，不执行高级调度）

```toml
flowbuilder = { version = "0.1.0", default-features = false, features = ["core"] }
```

#### 5. 全特性（探索/调试阶段）

```toml
flowbuilder = { version = "0.1.0", features = ["yaml", "runtime", "parallel", "retry", "perf-metrics", "detailed-logging"] }
```

### Import 策略建议

| 需求场景        | 推荐 imports                                                | 说明                  |
| --------------- | ----------------------------------------------------------- | --------------------- |
| 纯构建/串行流程 | `use flowbuilder::prelude::*;`                              | 不启用 runtime 子特性 |
| 高性能执行      | `use flowbuilder::runtime::prelude::*;`                     | 并行 / 重试 / 指标    |
| YAML 驱动       | `use flowbuilder::yaml::prelude::*;` + 视需要再引入 runtime | 分离解析与执行        |
| 最小二进制      | 仅 core，无 runtime/yaml                                    | 减少依赖与代码大小    |

> FAQ: 为什么不在 YAML prelude 里自动带 runtime? —— 避免无意拉入并行/重试/指标等额外逻辑，给予使用者显式控制权。
