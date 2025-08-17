# FlowBuilder

> 🚀 基于 Rust 的异步工作流 / 编排引擎：YAML 驱动 · 并行调度 · 可裁剪特性

[![Crates.io](https://img.shields.io/crates/v/flowbuilder.svg)](https://crates.io/crates/flowbuilder)
[![Documentation](https://docs.rs/flowbuilder/badge.svg)](https://docs.rs/flowbuilder)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## ✨ 特性概览

| 类别   | 能力        | 说明                                      |
| ------ | ----------- | ----------------------------------------- |
| 架构   | 分层设计    | Parser → Orchestrator → Executor 清晰职责 |
| 执行   | 并行/限流   | 自动阶段划分 + 信号量控制                 |
| 可靠   | 重试/超时   | 节点级策略配置                            |
| 可观测 | 指标/复杂度 | 执行耗时/任务统计 + 计划复杂度分析        |
| 调试   | 详细日志    | 按特性开启调试输出                        |
| 配置   | YAML 驱动   | 声明式定义 + 验证 + 计划预览              |
| 精简   | DSL 模式    | 仅使用构建器无需 YAML / Runtime           |

## 🚀 安装

默认启用 `core + yaml-runtime`：

```toml
[dependencies]
flowbuilder = { version = "0.1.1" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

最小（仅 DSL）：

```toml
flowbuilder = { version = "0.1.1", default-features = false, features = ["core"] }
```

## 🔌 Feature 体系

| 类别   | Feature          | 含义                                                  |
| ------ | ---------------- | ----------------------------------------------------- |
| 基础   | core             | 构建器 / 上下文                                       |
| 解析   | yaml             | YAML/JSON 解析 & 验证                                 |
| 执行   | runtime          | 编排 + 高级执行（含 parallel / retry / perf-metrics） |
| 组合   | yaml-runtime     | = yaml + runtime + 启用 yaml 对 runtime 的桥接        |
| 子特性 | parallel         | 阶段/节点并行调度                                     |
| 子特性 | retry            | 节点重试策略                                          |
| 子特性 | perf-metrics     | 执行统计（runtime 默认启用）                          |
| 子特性 | detailed-logging | 详细调试日志                                          |

常用组合：

| 场景      | features                      | 说明                |
| --------- | ----------------------------- | ------------------- |
| 仅构建    | core                          | DSL / 串行逻辑      |
| 最小执行  | runtime                       | 并行 + 重试 + 指标  |
| 仅解析    | yaml                          | 离线验证 / 静态检查 |
| YAML 执行 | yaml-runtime                  | 解析 + 编排 + 执行  |
| 调试      | yaml-runtime,detailed-logging | 加详细日志          |

perf-metrics 默认开启；禁用请自建 runtime-base（规划中）。

## 🧪 快速示例

```rust
use flowbuilder::prelude::*;            // DSL
use flowbuilder::yaml::prelude::*;      // 动态执行器
#[cfg(feature = "runtime")] use flowbuilder::runtime::prelude::*; // 高级执行

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let yaml = std::fs::read_to_string("workflow.yaml")?;
    let config = WorkflowLoader::from_yaml_str(&yaml)?;
    let mut exec = DynamicFlowExecutor::new(config)?;
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

> 提示：生产构建可关闭 `detailed-logging`，最小体积使用 `default-features = false`。

## 🏗️ 架构

```
┌──────────────┐    ┌────────────────┐    ┌────────────────┐
│  YAML Config │ -> │ Orchestrator   │ -> │   Executor      │
│ (Parser)     │    │ (Plan / Analyze) │ │ (Run / Metrics) │
└──────────────┘    └────────────────┘    └────────────────┘
```

核心组件：Parser / Orchestrator / Executor / SharedContext

## 📊 性能要点

零成本抽象 · Tokio 异步 · 并行调度 · 指标统计 · 内存友好

## 🔧 配置能力

任务定义 · 依赖/条件 · 重试 · 超时 · 变量注入 · 计划复杂度分析

## 📚 文档 & 示例

-   [快速入门](docs/quick-start-guide.md)
-   [架构设计](docs/architecture.md)
-   [API 参考](docs/api-reference.md)
-   [Chronetix 集成计划](docs/chronetix-integration-plan.md)
-   示例：`examples/new_architecture_demo.rs`

运行：

```bash
cargo test
cargo run --example new_architecture_demo
```

## 🌟 使用场景

微服务编排 / 数据管道 / CI&CD 自动化 / 业务流程 / API 工作流 / 批处理

## 🗺️ 路线规划

-   runtime-base 精细化子特性
-   分布式执行 (distributed / sharding)
-   扩展指标与可观测性

## 🤝 贡献

欢迎 Issue / PR 参与改进（参见 [CONTRIBUTING.md](CONTRIBUTING.md)）。

## 📄 许可证

Apache-2.0，详见 [LICENSE](LICENSE)。

---

用 ❤️ 打造的 Rust 工作流引擎
