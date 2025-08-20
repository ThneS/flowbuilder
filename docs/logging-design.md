# FlowBuilder 日志方案（tracing 统一化）

本文档设计 FlowBuilder 仓库统一日志方案，采用 Rust 社区主流的结构化日志库 tracing 与 tracing-subscriber，覆盖开发、测试、生产及分布式可观测性演进。

## 目标与范围

- 统一日志：替换 println!/eprintln! 为 tracing 宏（trace/debug/info/warn/error）。
- 结构化与可过滤：支持键值字段、动态过滤（环境变量/配置），便于排查与检索。
- 异步与并发友好：对异步执行、并行阶段与节点执行，使用 Span 关联上下文。
- 可演进：保留向 JSON、文件滚动、OpenTelemetry 导出扩展的空间（按 feature 开关）。
- 无侵入初始化：库不自行初始化日志，初始化只在二进制/示例/测试入口完成。

范围：flowbuilder 工作区各 crate（flowbuilder-core/context/runtime/yaml 及顶层 flowbuilder）。

## 日志级别与使用规范

- 宏映射：
  - println!(...) -> info!(...)
  - eprintln!(...) -> error!(...)
- 级别约定：
  - error：用户可见的失败、不可恢复错误。
  - warn：潜在问题、降级、重试等。
  - info：关键业务里程碑、阶段性结果、统计摘要。
  - debug：执行细节、分支判定、配置展开等。
  - trace：极细日志，热点循环或高频路径，默认关闭。
- 字段优先：倾向 info!(workflow_id=?id, phase=%name, ...; "消息") 形式，便于机器解析。

## 关键日志字段（统一字典）

适用于执行全链路（按语境选择子集）：
- 运行/工作流维度：
  - run_id（本次执行唯一 ID）、workflow_id、workflow_name、workflow_version
- 阶段/节点维度：
  - phase_id、phase_name、phase_index、node_id、node_name、node_type、action_id、action_type
- 结果与耗时：
  - success（bool）、error_kind、error_msg、retry_count、duration_ms
- 环境与配置：
  - env.ENVIRONMENT、log.level、parallel(enabled)、perf_metrics(enabled)

建议：
- 入口创建 run_id 并放入根 Span 的字段，子任务继承。
- 错误使用 error!(error=?err, ...)，必要时附加 backtrace（如 anyhow/eyre）。

## Span 设计与上下文传播

- 层级：
  - workflow（根 Span） -> phase -> node -> action
- 标注：关键异步函数使用 #[tracing::instrument(...)]，或在处理逻辑前手动 span!(...).in_scope(|| ...)。
- 并发：tokio::spawn 时使用 tracing-futures 的 Instrument trait 或 tokio::task::spawn(Instrumented future) 以继承父 Span。

示例（简化）：
```rust
#[tracing::instrument(skip(ctx), fields(run_id=%ctx.run_id, workflow=%workflow.name))]
async fn execute_workflow(ctx: &Context, workflow: &Workflow) -> Result<()> {
    for (i, phase) in workflow.phases.iter().enumerate() {
        let span = tracing::info_span!("phase", index=i, name=%phase.name);
        let _g = span.enter();
        // ... 执行节点
    }
    Ok(())
}
```

## 初始化与配置

- 仅在可执行入口初始化（examples、CLI、服务 main）：库 crate 不做全局初始化。
- 使用 tracing-subscriber + EnvFilter，支持 RUST_LOG 动态控制。
- 输出格式：
  - 开发默认：紧凑/pretty（彩色，人类友好）。
  - 生产建议：JSON（结构化，便于采集）。
- 环境变量（建议约定）：
  - RUST_LOG：标准过滤表达式（如：`info,flowbuilder_runtime=debug`）。
  - FB_LOG_FORMAT：`pretty` | `compact` | `json`（默认 pretty）。
  - FB_LOG_FILE：若设置则写入该文件（配合 tracing-appender non-blocking）。
  - FB_LOG_SPAN_EVENTS：`full` | `none`（是否记录 enter/exit 事件）。

参考初始化（入口 main 中）：
```rust
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = match std::env::var("FB_LOG_FORMAT").as_deref() {
        Ok("json") => fmt::layer().json().with_target(true).with_file(true).with_line_number(true),
        Ok("compact") => fmt::layer().compact(),
        _ => fmt::layer().pretty(),
    };

    // 可选：文件输出（rolling 按需开启，避免默认写盘）
    // let file_appender = tracing_appender::rolling::daily("logs", "flowbuilder.log");
    // let (nb, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
```

说明：如需文件或 JSON，请在二进制/示例中根据部署需求切换。库侧不持有 Guard。

## crate 与 feature 策略

- 依赖：
  - 已在 workspace.dependencies 中统一声明 tracing 与 tracing-subscriber。
  - 可选扩展：tracing-appender（文件滚动）、tracing-opentelemetry（otel 导出）。
- 特性开关：
  - 保持库代码中日志宏无 feature 限制（宏零开销被关掉时也应最小化成本）。
  - 若要精简依赖：各子 crate 可保留 `logger` 可选特性，仅在该特性启用时编译 tracing 依赖；但推荐工作区统一启用，降低条件编译分歧。
- 初始化入口：在 flowbuilder 顶层提供一个薄封装模块 `flowbuilder::logging::init()`（后续实现），供 examples/CLI 直接调用。

## 打印替换迁移指南

- 原则：一律替换 println!/eprintln!。
- 映射示例：
  - `println!("开始执行")` -> `tracing::info!("开始执行")`
  - `println!("阶段 {} 结束", i)` -> `tracing::info!(phase_index=i, "阶段结束")`
  - `eprintln!("错误: {}", err)` -> `tracing::error!(error=%err, "执行失败")`
- 大段多行打印（如计划预览）：
  - 若为人类可读摘要：info!(summary=%text)。
  - 若为调试细节/长文本：debug!(plan_pretty=%text) 或按行 trace!。

## 开发/测试/生产建议预设

- 开发：RUST_LOG=info,flowbuilder_runtime=debug；格式 pretty，少量 span 事件。
- CI/测试：RUST_LOG=info；格式 compact 或 json；失败用例保留日志工件。
- 生产：RUST_LOG=info（或 warn）；格式 json；按需开启文件/集中采集；禁止 trace 级别。

## 可观测性与后续演进（可选）

- OpenTelemetry：增加 feature `otel`，通过 tracing-opentelemetry 导出 trace/span 到 OTLP（Tempo/Jaeger/Zipkin）。
- 指标：在 runtime 中将关键统计以 info 字段输出，或引入 metrics crate（可选）结合 exporter。
- 采样：对 trace 级别或高频路径采用采样，降低成本。

## 性能与安全

- 性能：使用动态过滤（EnvFilter）关闭低级别日志时开销极低；热路径避免 format! 预计算。
- 隐私：避免输出敏感信息（token、密码、PII）；必要字段使用掩码或 hash。
- 稳定性：文件写入使用 non-blocking appender，防止阻塞执行线程。

## 验收清单（本次文档阶段）

- [x] 选型：tracing + tracing-subscriber
- [x] 级别/字段/Span 层级规范
- [x] 初始化与环境变量约定
- [x] 迁移与使用指南
- [x] 预设与演进方向

## 下一步实施计划

1) 入口初始化：在 examples/CLI 添加 tracing 初始化（开发默认 pretty，支持 RUST_LOG）。
2) 替换输出：批量替换各 crate 中 println!/eprintln! 为 tracing 宏，并补充关键字段。
3) 关键路径打点：对 workflow/phase/node/action 核心函数添加 #[instrument] 与 info_span。
4) 可选扩展：增加 `flowbuilder-logging` 模块/特性，封装 init；按需引入 tracing-appender 与 otel feature。
5) 文档与示例：在 README/quick-start 中加入日志配置说明与示例输出。
