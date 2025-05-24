# FlowBuilder

> A flexible, async Rust flow engine with conditional branching, context, retry, and subflows.
> 一个灵活的异步 Rust 流程引擎，支持条件分支、上下文、重试和子流程。

## ✨ Features | 特性

- Chain async steps: `.step(...)` | 链式异步步骤
- Shared context across steps | 步骤间共享上下文
- Conditional execution: `.step_if(...)` | 条件执行
- Retry & wait logic: `.wait_until(...)` | 重试和等待逻辑
- Error capturing without panicking | 错误捕获而不崩溃
- Named steps with logging | 带日志的命名步骤
- Nested subflows with `.subflow_if(...)` | 嵌套子流程

## 🧪 Example | 示例

```rust
#[tokio::main]
async fn main() -> Result<()> {
    FlowBuilder::new()
        .named_step("run", run)
        .named_step("check", check)
        .wait_until(|ctx| ctx.ok, Duration::from_secs(1), 3)
        .step_if(|ctx| ctx.ok, stop)
        .step_if(|ctx| ctx.ok, finish)
        .step_handle_error("finalize", finalize, |ctx, e| {
            ctx.errors.push(format!("{}", e));
            Ok(())
        })
        .run_all()
        .await?;

    Ok(())
}
```
