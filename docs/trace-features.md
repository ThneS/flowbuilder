# FlowBuilder 追踪功能更新

## 新增功能概览

### ✅ 已实现的功能

| 功能                          | 状态     | 说明                                                    |
| ----------------------------- | -------- | ------------------------------------------------------- |
| ✔ 条件跳转                    | 已实现   | 基于 `step_if()` 和 `FlowContext`                       |
| ✔ 上下文传递                  | 已实现   | 使用 `SharedContext = Arc<Mutex<FlowContext>>`          |
| ✅ **循环等待**               | **新增** | `step_wait_until()` - 直到条件满足才继续执行            |
| ✅ **每一步自定义名称、日志** | **新增** | `named_step()` 现在包含完整的步骤追踪                   |
| ✅ **错误不中断流程**         | **新增** | `step_continue_on_error()` - 错误写入 context，继续流程 |
| ✔ 嵌套子流程（分支流程）      | 已实现   | 支持流程嵌套执行，如 `subflow_if` / `parallel_steps`    |
| ✅ **Trace ID 追踪**          | **新增** | 每个流程都有唯一的 trace_id 用于追踪整个执行过程        |

### 🆕 新增的方法

#### 1. `step_continue_on_error()` - 错误不中断流程

```rust
pub fn step_continue_on_error<Fut, F>(self, name: &'static str, f: F) -> Self
```

-   执行步骤，如果出错则记录错误但不中断流程
-   错误信息记录在 `FlowContext.errors` 中
-   设置 `FlowContext.ok = false` 但继续执行后续步骤

#### 2. `step_wait_until()` - 循环等待条件

```rust
pub fn step_wait_until<Cond>(self, name: &'static str, cond: Cond, interval: Duration, max_retry: usize) -> Self
```

-   循环检查条件直到满足或达到最大重试次数
-   支持自定义检查间隔和最大重试次数
-   提供详细的等待过程日志

#### 3. `run_all_with_trace_id()` - 自定义追踪 ID

```rust
pub async fn run_all_with_trace_id(self, trace_id: String) -> Result<()>
```

-   使用自定义的 trace_id 执行流程
-   便于在分布式系统中追踪特定的流程实例

### 🔧 增强的功能

#### 1. FlowContext 增强

```rust
pub struct FlowContext {
    pub trace_id: String,           // 🆕 唯一追踪ID
    pub ok: bool,
    pub errors: Vec<String>,
    pub step_logs: Vec<StepLog>,    // 🆕 详细的步骤日志
    pub variables: HashMap<String, String>, // 🆕 变量存储
}
```

#### 2. 步骤日志记录

```rust
pub struct StepLog {
    pub step_name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub status: StepStatus,         // Running/Success/Failed/Skipped/Timeout
    pub error_message: Option<String>,
    pub trace_id: String,
}
```

#### 3. 完整的执行摘要

-   自动生成执行摘要，包括：
    -   总步骤数和各状态统计
    -   执行时间统计
    -   错误信息汇总
    -   变量状态快照

### 📝 使用示例

```rust
use flowbuilder::prelude::FlowBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flow = FlowBuilder::new()
        .named_step("initialization", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "0".to_string());
            Ok(())
        })
        .step_continue_on_error("risky_operation", |_ctx| async move {
            // 这个步骤可能失败，但不会中断流程
            anyhow::bail!("Something went wrong")
        })
        .step_wait_until(
            "wait_for_condition",
            |ctx| ctx.get_variable("ready") == Some(&"true".to_string()),
            Duration::from_millis(100),
            10
        )
        .named_step("finalization", |_ctx| async move {
            println!("Flow completed!");
            Ok(())
        });

    // 使用自定义 trace_id
    flow.run_all_with_trace_id("my-custom-trace-123".to_string()).await?;
    Ok(())
}
```

### 📊 输出示例

```
[trace_id:my-custom-trace-123] Starting flow execution with 4 steps
[trace_id:my-custom-trace-123] [step:initialization] starting...
[trace_id:my-custom-trace-123] setting variable counter = 0
[trace_id:my-custom-trace-123] [step:initialization] completed successfully in 152.157µs
[trace_id:my-custom-trace-123] [step:risky_operation] starting...
[trace_id:my-custom-trace-123] [step:risky_operation] failed after 142.063µs: Something went wrong
[trace_id:my-custom-trace-123] [step:wait_for_condition] starting...
[trace_id:my-custom-trace-123] [step:wait_for_condition] condition met on attempt 1
[trace_id:my-custom-trace-123] [step:wait_for_condition] completed successfully in 356.573µs

=== Flow Summary [trace_id: my-custom-trace-123] ===
Total steps: 4
Success: 3, Failed: 1, Skipped: 0, Timeout: 0
Errors: 1
  - [my-custom-trace-123] risky_operation: Something went wrong
Variables:
  counter = 0
==============================
```

### 🎯 与 workflow.yaml 的对应关系

新功能直接对应 `template/workflow.yaml` 中定义的特性：

-   **trace_id** ↔ 流程追踪和日志记录
-   **step_continue_on_error** ↔ `on_error` 错误处理
-   **step_wait_until** ↔ `while_util` 循环等待条件
-   **named_step** ↔ `action.name` 和 `action.description`
-   **variables** ↔ `outputs` 变量存储和传递
-   **timeout/retry** ↔ 已有的 `step_with_timeout` 和 `step_with_retry`

这些增强使 FlowBuilder 更加接近生产级别的工作流引擎，提供了完整的可观测性和错误处理能力。
