# FlowBuilder 入门指南

## 简介

FlowBuilder 是一个灵活的异步 Rust 流程引擎，它允许你以声明式的方式构建复杂的异步工作流。通过 FlowBuilder，你可以轻松地：

- 链式执行异步步骤
- 在步骤间共享上下文
- 实现条件执行
- 处理重试和等待逻辑
- 优雅地处理错误
- 创建嵌套子流程

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
flowbuilder = "0.1.0"
```

## 快速开始

### 基本用法

```rust
use flowbuilder::{FlowBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建一个新的流程
    FlowBuilder::new()
        // 添加一个命名步骤
        .named_step("初始化", init)
        // 添加条件步骤
        .step_if(|ctx| ctx.is_valid(), validate)
        // 添加等待逻辑
        .wait_until(|ctx| ctx.is_ready(), Duration::from_secs(1), 3)
        // 添加错误处理
        .step_handle_error("清理", cleanup, |ctx, e| {
            println!("发生错误: {}", e);
            Ok(())
        })
        // 运行所有步骤
        .run_all()
        .await?;

    Ok(())
}

// 步骤函数示例
async fn init(ctx: &mut Context) -> Result<()> {
    ctx.data.insert("value", 42);
    Ok(())
}

async fn validate(ctx: &mut Context) -> Result<()> {
    // 验证逻辑
    Ok(())
}
```

### 上下文共享

FlowBuilder 提供了强大的上下文共享机制：

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn step1(ctx: &mut Context) -> Result<()> {
    // 在上下文中存储数据
    ctx.insert("key", "value");
    Ok(())
}

async fn step2(ctx: &mut Context) -> Result<()> {
    // 从上下文中读取数据
    if let Some(value) = ctx.get::<&str>("key") {
        println!("从 step1 获取的值: {}", value);
    }
    Ok(())
}
```

### 条件执行

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn conditional_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .step_if(|ctx| ctx.get::<bool>("should_run").unwrap_or(false), run_step)
        .step_if(|ctx| ctx.get::<bool>("should_validate").unwrap_or(false), validate_step)
        .run_all()
        .await?;
    Ok(())
}
```

### 错误处理

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn error_handling_flow(ctx: &mut Context) -> Result<()> {
    FlowBuilder::new()
        .step(risky_operation)
        .step_handle_error("错误恢复", recovery_step, |ctx, e| {
            // 自定义错误处理逻辑
            ctx.errors.push(format!("操作失败: {}", e));
            Ok(())
        })
        .run_all()
        .await?;
    Ok(())
}
```

## 最佳实践

1. **命名步骤**
   - 始终为重要步骤提供有意义的名称
   - 使用清晰的命名约定

2. **错误处理**
   - 为每个可能失败的步骤提供错误处理
   - 在上下文中记录错误信息
   - 实现适当的恢复策略

3. **上下文管理**
   - 使用类型安全的方法访问上下文数据
   - 避免在上下文中存储过大的数据
   - 及时清理不再需要的数据

4. **条件执行**
   - 使用清晰的条件表达式
   - 避免复杂的嵌套条件
   - 考虑使用子流程处理复杂条件

## 下一步

- 查看 [高级用法](advanced-usage.md) 了解更多特性
- 阅读 [API 参考](api-reference.md) 获取详细文档
- 查看 [示例代码](../examples/) 获取更多使用示例