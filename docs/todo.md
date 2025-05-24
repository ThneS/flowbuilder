
### MVP TODO | MVP 待办
- [ ] step_named_async | 异步命名步骤
- [ ] parallel_subflow | 并行子流程
- [ ] context snapshot & restore | 上下文快照和恢复
- [ ] step_with_timeout | 带超时的步骤
- [ ] parallel_steps | 并行步骤
- [ ] snap && rollback | 快照和回滚
- [ ] TraceID | 追踪ID

## Feature Roadmap | 功能路线图

### MVP (Minimum Viable Product) | 最小可行产品
| Target | Recommended Features | 目标 | 推荐支持的功能 |
|--------|---------------------|------|--------------|
| MVP | ✅ Step/Condition/Context/Error Handling/Subflow | MVP（最小可用） | ✅ 步骤/条件/上下文/错误处理/子流程 |
| High Availability | ✅ Timeout Control/Logging/Trace ID/Nested Flow | 强可用 | ✅ 超时控制、日志、trace id、嵌套流程 |
| High Configurability | ✅ Config-Driven Build/Dynamic Switch/Global Error | 高可配置 | ✅ 配置驱动构建、动态 switch、全局错误 |
| Enterprise | ✅ Metrics/Audit/State Persistence/UI Designer | 企业级 | ✅ metrics、审计、状态持久化、UI设计器 |

### TODO Features | 待办功能
| Feature | Description | 功能 | 说明 |
|---------|-------------|------|------|
| Logging Integration | Hook into tracing/log libraries, generate standard log events | 日志集成 | 可挂接 tracing/log 库，生成标准日志事件 |
| Metrics | Expose Prometheus metrics like step_duration, success_count | metrics | 暴露 Prometheus 指标如 step_duration、success_count 等 |
| Context Persistence | Support context serialization and runtime recovery | 持久化上下文 | 支持序列化 context，恢复运行 |
| Audit Logs | Record state and duration for each step start/end | 审计日志 | 每个步骤开始/结束记录状态与耗时 |
| State Machine Support | Explicit flow state definitions (pending, running, failed) | 状态机支持 | 显式的流程状态定义（待定、运行中、失败等） |
| UI Visual Designer | Display flows graphically/support drag-and-drop definition | UI 可视化设计 | 将流程展示成图形/支持拖拽定义（可借助外部工具） |
| Dynamic Flow Loading | Build FlowBuilder from config/JSON | 动态流程结构加载 | 从配置/JSON 构建 FlowBuilder |
| Conditional Branching | Switch-case (multi-way branching) | 条件跳转 | switch-case（多路分支） |
| Global Error Handler | Unified fallback | 全局 error handler | 统一 fallback |

## Project Structure | 项目结构
```
flowbuilder/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── core.rs                   // MVP core functionality | MVP 核心功能
│   ├── context.rs               // FlowContext definition | FlowContext 定义
│   ├── features/
│   │   ├── timeout.rs            // High Availability: step timeout control | 强可用：step 超时控制
│   │   ├── logger.rs             // High Availability: logging integration | 强可用：日志集成
│   │   ├── trace.rs              // High Availability: trace id | 强可用：trace id
│   │   ├── switch.rs             // High Config: dynamic branching | 高配置：动态分支
│   │   ├── config_loader.rs      // High Config: JSON/config building | 高配置：JSON/配置构建
│   │   ├── global_error.rs       // High Config: unified error handling | 高配置：统一错误处理
│   └── utils.rs
└── tests/
    └── flowbuilder_tests.rs
```

## Feature Flags | 功能标志
```toml
[features]
default = ["mvp"]
mvp = []
strong = ["timeout", "logger", "trace"]
configurable = ["switch", "config_loader", "global_error"]

timeout = []
logger = ["log"]
trace = []
switch = []
config_loader = ["serde", "serde_json"]
global_error = []
```

## Data Flow | 数据流
| Data Object | Support | Recommended Method | 传递对象 | 支持 | 推荐方式 |
|------------|---------|-------------------|---------|------|---------|
| Synchronous Data | ✅ `FlowContext.insert/get` | | 同步传递数据 | ✅ `FlowContext.insert/get` | |
| Cross-step Parameter Sharing | ✅ Default support | | 跨 step 参数共享 | ✅ 默认支持 | |
| Cross-subflow Data Sharing | ✅ Same context by default | | 跨子流程共享数据 | ✅ 默认同一上下文 | |
| Type Safety | ✅ Extensible wrapper | | 类型安全 | ✅ 可封装扩展 | |
| Multi-tenant/Isolated Context | ✅ Multiple contexts support | | 多租户/隔离上下文 | ✅ 多个上下文支持 | |
