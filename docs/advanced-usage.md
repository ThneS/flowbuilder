# FlowBuilder 高级用法

## 子流程

FlowBuilder 支持创建嵌套的子流程，这对于组织复杂的业务逻辑非常有用。

### 基本子流程

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn main_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .step(initialization)
        .subflow("处理数据", |ctx| async move {
            FlowBuilder::new()
                .step(process_data)
                .step(validate_data)
                .run_all()
                .await
        })
        .step(finalization)
        .run_all()
        .await?;
    Ok(())
}
```

### 条件子流程

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn conditional_subflow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .subflow_if(
            |ctx| ctx.get::<bool>("should_process").unwrap_or(false),
            |ctx| async move {
                FlowBuilder::new()
                    .step(process_data)
                    .step(validate_data)
                    .run_all()
                    .await
            },
        )
        .run_all()
        .await?;
    Ok(())
}
```

## 超时控制

FlowBuilder 提供了超时控制机制，可以防止步骤执行时间过长。

```rust
use flowbuilder::{FlowBuilder, Context, Result};
use std::time::Duration;

async fn timeout_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .step_with_timeout(
            "长时间运行的操作",
            long_running_operation,
            Duration::from_secs(5),
        )
        .step_handle_error("超时处理", handle_timeout, |ctx, e| {
            if e.is_timeout() {
                println!("操作超时");
            }
            Ok(())
        })
        .run_all()
        .await?;
    Ok(())
}
```

## 并行执行

FlowBuilder 支持并行执行多个步骤，提高执行效率。

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn parallel_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .parallel_steps(vec![
            ("任务1", task1),
            ("任务2", task2),
            ("任务3", task3),
        ])
        .step(collect_results)
        .run_all()
        .await?;
    Ok(())
}
```

## 上下文快照

FlowBuilder 支持创建上下文的快照，并在需要时恢复。

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn snapshot_flow(ctx: &mut Context) -> Result<()> {
    // 创建快照
    let snapshot = ctx.snapshot()?;

    FlowBuilder::new()
        .step(risky_operation)
        .step_handle_error("恢复", |ctx, e| {
            // 发生错误时恢复快照
            ctx.restore(&snapshot)?;
            Ok(())
        })
        .run_all()
        .await?;
    Ok(())
}
```

## 自定义步骤类型

你可以创建自定义的步骤类型来扩展 FlowBuilder 的功能。

```rust
use flowbuilder::{Step, Context, Result};

struct RetryStep<F> {
    name: String,
    func: F,
    max_retries: usize,
    delay: Duration,
}

impl<F> Step for RetryStep<F>
where
    F: Fn(&mut Context) -> Result<()> + Send + Sync,
{
    async fn execute(&self, ctx: &mut Context) -> Result<()> {
        let mut retries = 0;
        while retries < self.max_retries {
            match (self.func)(ctx) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retries += 1;
                    if retries == self.max_retries {
                        return Err(e);
                    }
                    tokio::time::sleep(self.delay).await;
                }
            }
        }
        Ok(())
    }
}
```

## 性能优化

### 1. 上下文优化

```rust
use flowbuilder::{FlowBuilder, Context, Result};

// 使用类型化的上下文键
#[derive(Debug, Clone, Copy)]
struct UserId(u64);

impl Context {
    fn set_user_id(&mut self, id: UserId) {
        self.insert("user_id", id.0);
    }

    fn get_user_id(&self) -> Option<UserId> {
        self.get::<u64>("user_id").map(UserId)
    }
}
```

### 2. 内存管理

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn memory_efficient_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .step(|ctx| {
            // 处理完成后清理大型数据
            ctx.remove("large_data");
            Ok(())
        })
        .run_all()
        .await?;
    Ok(())
}
```

## 调试和日志

FlowBuilder 提供了内置的日志支持，可以帮助你调试流程执行。

```rust
use flowbuilder::{FlowBuilder, Context, Result};
use tracing::{info, error};

async fn logged_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .named_step("开始", |ctx| {
            info!("流程开始执行");
            Ok(())
        })
        .step_handle_error("错误", |ctx, e| {
            error!("发生错误: {}", e);
            Ok(())
        })
        .run_all()
        .await?;
    Ok(())
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use flowbuilder::Context;

    #[tokio::test]
    async fn test_basic_flow() {
        let mut ctx = Context::new();
        let result = basic_flow(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(ctx.get::<i32>("value"), Some(42));
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_workflow() {
        let mut ctx = Context::new();
        let result = complete_workflow(&mut ctx).await;
        assert!(result.is_ok());
        // 验证所有预期的状态
    }
}
```

## 最佳实践

1. **错误处理**
   - 使用自定义错误类型
   - 实现适当的错误转换
   - 保持错误信息的具体性

2. **性能考虑**
   - 避免在上下文中存储过大的数据
   - 使用并行执行提高性能
   - 适当使用超时控制

3. **可维护性**
   - 使用有意义的步骤名称
   - 保持步骤函数的单一职责
   - 适当使用子流程组织代码

4. **测试**
   - 为每个步骤编写单元测试
   - 编写集成测试验证完整流程
   - 使用模拟对象测试外部依赖