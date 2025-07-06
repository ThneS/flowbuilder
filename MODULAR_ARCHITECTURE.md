# FlowBuilder - Modular Architecture

FlowBuilder 现在采用模块化架构，使用 features 和多个 crate 来组织代码。

## 架构概览

```
flowbuilder/
├── flowbuilder/          # 主crate，重新导出所有功能
├── flowbuilder-core/     # 核心流程构建功能
├── flowbuilder-context/  # 上下文管理
├── flowbuilder-macros/   # 宏定义
├── flowbuilder-logger/   # 日志和追踪
└── flowbuilder-runtime/  # 高级运行时功能
```

## 可用的 Features

### 核心 Features

-   `core` (默认): 基础流程构建功能
-   `macros`: 过程宏支持，简化流程定义
-   `logger`: 日志和追踪支持
-   `runtime`: 高级运行时功能（并行执行、调度等）
-   `full`: 启用所有功能

### 遗留 Features（向后兼容）

-   `mvp`: 等同于`core`
-   `strong`: 等同于`logger`

## 使用方法

### 基础使用

```toml
[dependencies]
flowbuilder = "0.0.2"
```

```rust
use flowbuilder::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flow = FlowBuilder::new()
        .step(|ctx| async move {
            println!("Hello, FlowBuilder!");
            Ok(())
        })
        .build();

    flow.execute().await?;
    Ok(())
}
```

### 使用宏功能

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["macros"] }
```

```rust
use flowbuilder::prelude::*;
use flowbuilder_macros::named_step;

#[named_step("my_step")]
async fn my_step(ctx: SharedContext) -> anyhow::Result<()> {
    println!("Step with automatic logging!");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flow = FlowBuilder::new()
        .step(|ctx| my_step(ctx))
        .build();

    flow.execute().await?;
    Ok(())
}
```

### 使用日志功能

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["logger"] }
```

```rust
use flowbuilder::prelude::*;
use flowbuilder_logger::Logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化追踪
    Logger::init_tracing();

    let flow = FlowBuilder::new()
        .named_step("logged_step", |ctx| async move {
            println!("This step will be automatically logged");
            Ok(())
        })
        .build();

    let result = flow.execute().await?;

    // 记录执行摘要
    let logger = Logger::new();
    logger.log_flow_summary(&result);

    Ok(())
}
```

### 使用运行时功能

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["runtime"] }
```

```rust
use flowbuilder::prelude::*;
use flowbuilder_runtime::{FlowBuilderExt, ScheduleOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 并行执行
    let result = FlowBuilder::new()
        .step(|ctx| async move {
            println!("Parallel step 1");
            Ok(())
        })
        .step(|ctx| async move {
            println!("Parallel step 2");
            Ok(())
        })
        .execute_parallel()
        .execute()
        .await?;

    // 调度执行
    let result = FlowBuilder::new()
        .step(|ctx| async move {
            println!("Scheduled step");
            Ok(())
        })
        .schedule(ScheduleOptions::once_after(Duration::from_secs(1)))
        .execute()
        .await?;

    Ok(())
}
```

### 使用所有功能

```toml
[dependencies]
flowbuilder = { version = "0.0.2", features = ["full"] }
```

## 独立使用各个 Crate

你也可以独立使用各个 crate：

```toml
[dependencies]
flowbuilder-core = "0.0.2"
flowbuilder-macros = "0.0.2"
flowbuilder-logger = { version = "0.0.2", optional = true }
```

## 特性对比

| Feature   | 功能           | 依赖                                      |
| --------- | -------------- | ----------------------------------------- |
| `core`    | 基础流程构建   | `flowbuilder-core`, `flowbuilder-context` |
| `macros`  | 过程宏支持     | `flowbuilder-macros`                      |
| `logger`  | 日志追踪       | `flowbuilder-logger`, `tracing`           |
| `runtime` | 并行执行、调度 | `flowbuilder-runtime`                     |

## 架构优势

1. **模块化**: 每个功能都在独立的 crate 中，便于维护和测试
2. **可选依赖**: 用户可以根据需要选择功能，减少编译时间和二进制大小
3. **向后兼容**: 保持原有 API 不变，同时提供新的功能
4. **扩展性**: 容易添加新的功能模块
5. **职责分离**: 每个 crate 都有明确的职责范围
