# API 参考 (API Reference)

> 本文件提供高频使用类型与入口函数概览；详细文档请参见 docs.rs: https://docs.rs/flowbuilder

## 主入口 (Facade)

```rust
use flowbuilder::prelude::*;
```

导出：

-   FlowBuilder / FlowBuilderStep
-   ExecutionPlan / ExecutionPhase
-   SharedContext / FlowContext (来自 context crate)
-   EnhancedExecutor / EnhancedOrchestrator (feature = "runtime")
-   DynamicFlowExecutor (feature = "yaml")

## 构建流程 (Fluent API)

```rust
FlowBuilder::new()
    .named_step("init", |_ctx| async move { Ok(()) })
    .parallel_steps(vec![/* 子流程 */])
    .step_with_retry("io", 3, Duration::from_millis(200), handler)
    .run_all()
    .await?;
```

常用方法：

-   named_step(id, handler)
-   step_with_retry(id, retries, delay, handler)
-   step_with_timeout(id, duration, handler)
-   step_continue_on_error(id, handler)
-   step_handle_error(id, work_handler, error_handler)
-   parallel_steps(subflows)
-   parallel_steps_with_join(id, subflows)
-   run_all()
-   run_all_with_timeout(dur)
-   run_all_with_trace_id(trace_id)

## 动态执行 (YAML)

```rust
use flowbuilder_yaml::prelude::*;
use tracing::info;
fn init_logging() { flowbuilder::logging::init(); }
let yaml = std::fs::read_to_string("workflow.yaml")?;
let mut exec = DynamicFlowExecutor::from_yaml(&yaml)?;
let ctx = std::sync::Arc::new(tokio::sync::Mutex::new(flowbuilder_context::FlowContext::default()));
let result = exec.execute(ctx).await?;
info!(success = result.success);
```

## 增强执行 (Runtime)

```rust
use flowbuilder_runtime::prelude::*;
let plan = /* 构建 ExecutionPlan */;
let executor = EnhancedTaskExecutor::new(Default::default());
let result = executor.execute_plan(plan).await?;
```

配置结构：

```rust
pub struct ExecutorConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout: u64,
}
```

子特性 (runtime)：`parallel` / `retry` / `perf-metrics` / `detailed-logging`

## 错误分类（建议）

| 分类           | 描述             | 示例                   |
| -------------- | ---------------- | ---------------------- |
| ConfigError    | 配置/解析失败    | 缺失字段、非法枚举     |
| BuildError     | 构建执行计划失败 | 依赖环、节点缺失       |
| ExecutionError | 运行时执行失败   | 任务 panic/anyhow 错误 |
| RetryExhausted | 重试耗尽         | 达到最大重试次数       |
| Timeout        | 超时             | 步骤/全局超时          |

后续将统一错误枚举暴露到 facade。

## 结果与指标

-   ExecutionResult: success, total_duration, nodes_executed
-   StepLog / Trace (WIP): 追踪单步执行耗时与状态

## 特性矩阵

| Feature          | 默认        | 说明                 |
| ---------------- | ----------- | -------------------- |
| core             | ✔           | 基础抽象与构建器     |
| runtime          | ✔           | 增强执行/编排        |
| yaml             | ✔           | YAML/JSON 动态加载   |
| parallel         | ✔ (runtime) | 并行执行、信号量控制 |
| retry            | ✔ (runtime) | 节点重试策略         |
| perf-metrics     | ✔ (runtime) | 执行统计与耗时采集   |
| detailed-logging | ✘           | 详细调试日志         |

最小化：仅开启 `core` + `runtime` (去除其他子特性) 或完全关闭 `runtime` 仅用基础构建器。

---

### Import 组合示例

```rust
// 核心 (最小)
use flowbuilder::prelude::*;

// 高级执行能力 (需要 runtime)
use flowbuilder::runtime::prelude::*;

// YAML 动态加载 (需要 yaml)
use flowbuilder::yaml::prelude::*;

// YAML + 高级执行：同时导入两者
use flowbuilder::yaml::prelude::*;
use flowbuilder::runtime::prelude::*;
```

> YAML crate 不再隐式导出 runtime 类型，保证 feature 边界清晰。

补充或改进请提交 Issue。
