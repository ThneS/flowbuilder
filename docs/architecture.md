# 架构设计 (Architecture)

本文件概述 FlowBuilder 的分层架构与各 crate 之间的职责边界，帮助你根据需求最小化依赖与特性启用。

## 分层与 Crate 职责

| 层级        | Crate               | 主要职责                             | 可选性                     |
| ----------- | ------------------- | ------------------------------------ | -------------------------- |
| Context     | flowbuilder-context | 运行时共享上下文与变量管理           | 必需                       |
| Core        | flowbuilder-core    | 流程/节点/执行计划抽象与构建         | 必需                       |
| Runtime     | flowbuilder-runtime | 增强型调度、并发执行、重试、性能统计 | 可选 (feature = "runtime") |
| YAML Loader | flowbuilder-yaml    | YAML/JSON 配置解析与动态执行         | 可选 (feature = "yaml")    |
| Facade      | flowbuilder         | 统一对外 API 与 feature 聚合         | 入口                       |

## 模块交互

```
YamlConfigParser --> EnhancedOrchestrator --> ExecutionPlan --> EnhancedExecutor
         |                                              ^             |
         |----------- DynamicFlowExecutor ---------------+-------------|
```

## 特性 (Features) 规划

当前主 crate 默认启用 `core`, `runtime`, `yaml`。未来可拆分：

-   `perf-metrics`：性能统计与执行时间收集
-   `detailed-logging`：细粒度调试日志
-   `retry`：高级重试与退避策略
-   `parallel`：并行/信号量控制（禁用时退化为串行）

## 数据结构核心

-   ExecutionPlan：分阶段的节点计划 (phase -> nodes)
-   FlowContext：线程安全共享状态 (RwLock/Mutex 包裹内部 HashMap)
-   EnhancedTaskExecutor：带并发、重试与监控的执行器

## 关闭可选功能的收益

| 功能             | 典型开销             | 关闭收益                 |
| ---------------- | -------------------- | ------------------------ |
| perf-metrics     | 额外时间戳采集与聚合 | 降低系统调用/时钟读取    |
| detailed-logging | 大量字符串拼接与 IO  | 降低日志体积与格式化成本 |
| retry            | 状态跟踪与延迟等待   | 精简控制流               |
| parallel         | 信号量/任务调度      | 简化调度路径，利于嵌入式 |

## 下一步演进

1. 引入上述精细 feature 并在 `runtime` crate 内部以 `cfg` 分离
2. 提供 `cargo feature` 范例矩阵与 README 表格
3. 基准测试（criterion）覆盖：串行 vs 并行、有/无 metrics、有/无 logging

---

欢迎根据业务诉求提交 issue 讨论新的拆分维度。

## 特性组合与最小化策略

| 使用场景  | 推荐 features                                    | 说明                                       |
| --------- | ------------------------------------------------ | ------------------------------------------ |
| 最小核心  | core                                             | 仅构建与串行执行抽象                       |
| 高级执行  | core + runtime (+ parallel/retry/perf-metrics)   | 并行/重试/指标可按需裁剪                   |
| YAML 动态 | core + yaml (+ runtime)                          | 解析与执行解耦；不再自动 re-export runtime |
| 全量调试  | yaml + runtime + detailed-logging + perf-metrics | 研发期可观测性最大化                       |

示例（极简 + 并行执行）：

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core", "runtime", "parallel"] }
```

示例（YAML + 高级执行 + 关闭指标/日志）：

```toml
flowbuilder = { version = "0.1.0", features = ["yaml", "runtime", "parallel", "retry"] }
```

Import 建议：

```rust
// 核心
use flowbuilder::prelude::*;
// 高级执行
use flowbuilder::runtime::prelude::*;
// YAML 解析（如需同时执行器，显式再引入 runtime）
use flowbuilder::yaml::prelude::*;
```
