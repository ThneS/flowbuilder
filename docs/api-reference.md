# FlowBuilder API 参考

## 核心类型

### FlowBuilder

主要的流程构建器类型，用于创建和执行工作流。

```rust
pub struct FlowBuilder {
    steps: Vec<Box<dyn Step>>,
    context: Context,
}
```

#### 方法

##### new
```rust
pub fn new() -> Self
```
创建一个新的 FlowBuilder 实例。

##### step
```rust
pub fn step<F>(self, func: F) -> Self
where
    F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static
```
添加一个基本步骤。

##### named_step
```rust
pub fn named_step<F>(self, name: &str, func: F) -> Self
where
    F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static
```
添加一个带名称的步骤。

##### step_if
```rust
pub fn step_if<F, P>(self, predicate: P, func: F) -> Self
where
    F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    P: Fn(&Context) -> bool + Send + Sync + 'static
```
添加一个条件步骤。

##### step_handle_error
```rust
pub fn step_handle_error<F, H>(self, name: &str, func: F, handler: H) -> Self
where
    F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    H: Fn(&mut Context, Error) -> Result<()> + Send + Sync + 'static
```
添加一个带错误处理的步骤。

##### wait_until
```rust
pub fn wait_until<P>(self, predicate: P, interval: Duration, max_attempts: usize) -> Self
where
    P: Fn(&Context) -> bool + Send + Sync + 'static
```
添加一个等待条件满足的步骤。

##### subflow
```rust
pub fn subflow<F>(self, name: &str, flow: F) -> Self
where
    F: FnOnce(Context) -> Future<Output = Result<Context>> + Send + 'static
```
添加一个子流程。

##### subflow_if
```rust
pub fn subflow_if<P, F>(self, predicate: P, flow: F) -> Self
where
    P: Fn(&Context) -> bool + Send + Sync + 'static,
    F: FnOnce(Context) -> Future<Output = Result<Context>> + Send + 'static
```
添加一个条件子流程。

##### run_all
```rust
pub async fn run_all(self) -> Result<()>
```
运行所有步骤。

### Context

流程上下文，用于在步骤间共享数据。

```rust
pub struct Context {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
    errors: Vec<String>,
}
```

#### 方法

##### new
```rust
pub fn new() -> Self
```
创建一个新的上下文实例。

##### insert
```rust
pub fn insert<T: 'static + Send + Sync>(&mut self, key: &str, value: T)
```
在上下文中插入一个值。

##### get
```rust
pub fn get<T: 'static>(&self, key: &str) -> Option<&T>
```
从上下文中获取一个值。

##### remove
```rust
pub fn remove<T: 'static>(&mut self, key: &str) -> Option<T>
```
从上下文中移除一个值。

##### snapshot
```rust
pub fn snapshot(&self) -> Result<ContextSnapshot>
```
创建上下文的快照。

##### restore
```rust
pub fn restore(&mut self, snapshot: &ContextSnapshot) -> Result<()>
```
从快照恢复上下文。

### Error

自定义错误类型。

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("步骤执行失败: {0}")]
    StepExecution(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("上下文错误: {0}")]
    Context(String),

    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}
```

## 特性

### Step

步骤特征，用于定义可执行的步骤。

```rust
pub trait Step: Send + Sync {
    async fn execute(&self, ctx: &mut Context) -> Result<()>;
}
```

### ContextSnapshot

上下文快照特征，用于保存和恢复上下文状态。

```rust
pub trait ContextSnapshot: Send + Sync {
    fn restore(&self, ctx: &mut Context) -> Result<()>;
}
```

## 常量

### 默认值

```rust
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_RETRY_INTERVAL: Duration = Duration::from_secs(1);
pub const DEFAULT_MAX_RETRIES: usize = 3;
```

## 类型别名

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## 错误处理

### 错误类型

FlowBuilder 使用自定义的 `Error` 枚举类型来处理各种错误情况：

- `StepExecution`: 步骤执行失败
- `Timeout`: 操作超时
- `Context`: 上下文相关错误
- `Other`: 其他类型的错误

### 错误转换

FlowBuilder 提供了从标准库错误类型到自定义错误类型的转换：

```rust
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Other(err.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Other(err.into())
    }
}
```

## 日志和追踪

FlowBuilder 集成了 `tracing` 库用于日志记录和追踪：

```rust
use tracing::{debug, info, warn, error};

// 在步骤中使用
async fn logged_step(ctx: &mut Context) -> Result<()> {
    info!("开始执行步骤");
    // ... 执行逻辑 ...
    debug!("步骤执行完成");
    Ok(())
}
```

## 示例

### 基本用法

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn example() -> Result<()> {
    let mut ctx = Context::new();
    ctx.insert("value", 42);

    FlowBuilder::new()
        .named_step("初始化", |ctx| {
            println!("初始化步骤");
            Ok(())
        })
        .step_if(|ctx| ctx.get::<i32>("value") > 40, |ctx| {
            println!("条件步骤");
            Ok(())
        })
        .run_all()
        .await?;

    Ok(())
}
```

### 错误处理

```rust
use flowbuilder::{FlowBuilder, Context, Result, Error};

async fn error_handling_example() -> Result<()> {
    FlowBuilder::new()
        .step_handle_error("错误处理", |ctx| {
            Err(Error::StepExecution("步骤失败".into()))
        }, |ctx, e| {
            println!("处理错误: {}", e);
            Ok(())
        })
        .run_all()
        .await?;

    Ok(())
}
```

### 子流程

```rust
use flowbuilder::{FlowBuilder, Context, Result};

async fn subflow_example() -> Result<()> {
    FlowBuilder::new()
        .subflow("子流程", |ctx| async move {
            FlowBuilder::new()
                .step(|ctx| {
                    ctx.insert("sub_value", 100);
                    Ok(())
                })
                .run_all()
                .await?;
            Ok(ctx)
        })
        .run_all()
        .await?;

    Ok(())
}