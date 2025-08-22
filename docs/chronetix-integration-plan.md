# Flowbuilder × Chronetix 集成计划（对齐 Envelope/EventBus/DataPort）

-   Status: Draft
-   Version: 0.2.0
-   Date: 2025-08-17

参考：`docs/Chronetix/FLOWBUILDER_INTEGRATION.md`

## 1. 目标（Goals）

-   编排与执行解耦：Flowbuilder 负责 DAG/DSL 编排与静态分析；Chronetix 负责高精度时间驱动与节点执行。
-   统一消息边界：以 Envelope（src/dst/topic/corr_id/deadline_ns/...）为稳定契约；控制面走 EventBus，数据面走 DataPort。
-   平滑演进：先单进程 inproc，再 IPC（UDS/CBOR + SHM ring），最终可演进到网络（QUIC + CBOR），上层图编排无需改动。
-   全链路可观测：延迟/jitter/deadline miss/背压等指标与 Telemetry 事件可回流到 Flowbuilder 面板。

非目标（本阶段不做）：多租权限治理与跨集群一致性（留接口，原则上由宿主系统治理）。

## 2. 方法（Approach）

以“编译-运行”分离为主：

-   Compile（Flowbuilder）

    -   从 YAML/DSL 解析工作流图；校验依赖/条件；计算阶段与并发界限。
    -   产物：PluginManifest[]、routes（topic/port 映射）、schemas（content_type=application/cbor, schema_ver）。
    -   入口：`FlowAdapter.compile(graph) -> { manifests, routes, schemas }`。

-   Run（Chronetix）
    -   `NodeRunner: run(input)->output` 由 Chronetix Executor 托管，TimeManager 负责 tick/deadline/priority。
    -   背压与 credit 由 DataPort 反馈并影响上游 rate；WouldBlock 用于快速退避。

部署形态（演进阶梯）：

1. Inproc（单进程）：进程内 EventBus/DataPort 模拟；最快落地 PoC。
2. IPC：IpcEventBus（UDS+CBOR）+ ShmDataPort（环形共享内存）；等价功能，延迟可控。
3. 网络：QUIC + CBOR；单跳远端算子与重连/超时/错误分类稳定。

## 3. 架构与映射（Architecture & Mapping）

-   模型映射

    -   节点/算子 → Chronetix 插件/任务（WASM/dylib/进程隔离均可）。
    -   边/通道 → EventBus 主题（控制面）+ DataPort 通道（数据面）。
    -   消息 → Envelope（携带 src/dst/topic/corr_id/deadline_ns/priority/schema_ver 等）。

-   时序与 SLA

    -   窗口/水位线/超时 → deadline_ns + priority；Tick/定时器由 TimeManager 产生。
    -   自适应调度（p95/hysteresis）在负载波动时降低 deadline miss。

-   背压与速率

    -   sink 侧 credit 回传上游；达到阈值触发 WouldBlock；可选限速插件以阈值/EMA 控制输入速率。

-   安全与隔离
    -   插件形态可为 WASM（WIT ABI）、dylib 或进程；IPC/网络接口遵循 RFC_DISTRIBUTED_INTERFACE。

## 4. 契约与扩展（Contracts & Schema）

-   Envelope：按 `docs/rfc/draft/RFC_DISTRIBUTED_INTERFACE.md` §2。
-   EventBus API：publish/request/subscribe（控制面）。
-   DataPort API：acquire_credit/send_frame/on_frame（数据面）。
-   FlowAdapter：

    -   `compile(graph) -> { PluginManifest[], routes, schemas }`。
    -   输出应包含：
        -   每个节点的 `plugin_id`/`artifact`/`capabilities`（是否需要 DataPort、credit 阈值等）。
        -   每条边的 `topic`/`port`/`content_type`/`schema_ver`。
        -   节点运行参数：`deadline_ns`/`priority`/重试策略（如由 Chronetix 托管）。

-   YAML/DSL 扩展（Flowbuilder 侧）
    -   node.plugin: { kind: wasm|dylib|process, artifact, capabilities }
    -   io.schema: { content_type: application/cbor, schema_ver: "v1" }
    -   qos: { deadline_ns, priority, retry, max_inflight, credit_high, credit_low }
    -   bus/dataport: { topic, port, buffer, watermark }

## 5. 里程碑（Plan & Milestones）

-   M1 PoC（inproc，单进程）

    1. 最小 DAG：Source → Map → Sink。
    2. EventBus 传控制消息，DataPort 传数据帧。
    3. 超时映射为 deadline_ns；导出 p95/p99 与 miss 计数。
       验收：RTT p95 可观测；背压从 sink 向上游生效；无死锁/泄漏。

-   M2 IPC（UDS/CBOR + SHM Ring）

    1. EventBus → IpcEventBus（UDS+CBOR）。
    2. DataPort → ShmDataPort（环形共享内存 + 事件通知）。
       验收：功能等价；p95 延迟退化 ≤ inproc 的 3x；稳定无资源泄漏。

-   M3 网络（QUIC + CBOR）
    1. 单跳远端算子；断线重连/超时/错误分类完善。
       验收：Envelope 编解码兼容；断线恢复正确；指标齐全。

## 6. TODO（Backlog，含 Flowbuilder 侧工作）

-   [ ] 在 `flowbuilder-core/examples` 添加 `flow_poc.rs`：读取简化 DAG → 生成 `PluginManifest + routes + schemas`（inproc PoC）。
-   [ ] 新建适配 crate：`chronetix-flowbridge`（或 `flowbuilder-chronetix`）：定义 `FlowAdapter/NodeRunner` traits 与 inproc 实现。
-   [ ] IpcEventBus/ShmDataPort 接口对齐（按 RFC）：落原型与冒烟测试。
-   [ ] 指标桥接：将 Chronetix `MetricsRegistry` 导入 Flowbuilder 面板（或导出到统一 metrics）。
-   [ ] Schema registry：保持 content_type/schema_ver 的兼容策略与样例。
-   [ ] 文档联动：在两个项目 README 互链；标注部署矩阵（inproc / IPC / 网络）。
-   [ ] YAML/DSL 扩展字段定义与验证（`flowbuilder-yaml`）：plugin/qos/bus/dataport/schema 等。
-   [ ] Planner 增强：在编译期计算 deadline/priority 的默认值与继承规则；生成 manifests。
-   [ ] 单元/集成测试：最小 DAG、背压、deadline miss、重试与超时、断线恢复（M3）。

## 7. 质量门槛（Validation & SLO）

-   功能：依赖/条件、并发、重试、超时、上下文、背压全覆盖。
-   性能：在 M1/M2/M3 分别记录 p50/p95/p99 与 miss；IPC 退化 ≤ 3x。
-   稳定：无死锁/泄漏；DataPort recycle 正确；WouldBlock 路径可观测。
-   可观测：指标和 Telemetry 事件可回流至 Flowbuilder；日志按 feature 控噪。

## 8. 风险与开放问题

-   Envelope 版本与 schema 兼容：需与 Chronetix RFC 锁定版本并提供升级策略。
-   运行归属：重试/超时由谁托管（建议 Chronetix）；Flowbuilder 侧仅描述策略。
-   多实例并发与配额：是否需要跨进程限流/锁配额（建议由 Chronetix 或宿主协调）。

## 9. 版本与开关（Features）

-   Flowbuilder 侧：`yaml-runtime`（默认含 `perf-metrics`）、`detailed-logging`（调试）。
-   集成特性：`chronetix-integration`（启用 FlowAdapter/NodeRunner 与指标桥接）。
-   形态开关：`inproc` / `ipc` / `network`（构建与运行参数）。

## 10. 下一步（Next Steps）

-   对齐 Chronetix RFC 细节（Envelope 字段、EventBus/DataPort API）。
-   拉起 M1 PoC：提交 `flow_poc.rs` 与 inproc 适配；补最小使用文档与冒烟测试。
-   评审通过后推进 M2 IPC 与 YAML/DSL 扩展落地。

—

维护者：Flowbuilder 团队 · 参照 `docs/Chronetix/FLOWBUILDER_INTEGRATION.md` 持续更新

---

## 11. 相关文档（Related Docs）

- 本仓：`docs/chronetix/RESPONSIBILITIES_AND_APIS.md` — Flowbridge（Flowbuilder 侧）职责与公共接口
- 关联仓：Chronetix `docs/integration/FLOWBRIDGE_API_CONTRACT.md` — 接口边界与关键 API（总览）
- 关联仓：Chronetix `docs/integration/ENCODING_OPTIONS.md` — 控制面编码选项
