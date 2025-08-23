# Flowbuilder × Chronetix — 职责与公共接口（Flowbuilder 侧）

- Status: Draft
- Version: 0.1.0
- Date: 2025-08-22

## 1. 职责边界

Flowbuilder（编排/装配/计划）
- DAG/DSL 解析与静态分析；依赖与兼容性校验。
- 组件装配与能力治理：能力发现（WIT/Component Model 自描述）与兼容性检查。
- 编译期产物：`FlowAdapter.compile(graph) -> { manifests, routes, schemas }`。
- Schema 与策略：content_type/schema_ver 规划与兼容；将窗口/超时等策略下沉为 QoS（deadline/priority/retry/max-inflight/credit 阈值）。
- 编排 EventBus/TimeManager：通过平台节点或节点 bindings 产生 timers/subscriptions/publications。
- 测试与模拟：Mock Source/Sink、回放、背压注入。

Chronetix（运行/调度/传输）
- 执行器与调度：Executor + TimeManager；NodeRunner 宿主。
- 控制面与数据面：EventBus（控制面）与 DataPort（数据面）。
- 背压与资源治理：credit/水位线/WouldBlock；限速/熔断/降级。
- 可观测：MetricsRegistry；p95/p99、deadline miss、阻塞时长等。

## 2. 公共接口（Flowbuilder 提供/使用）

- FlowAdapter.compile（提供）：将 DAG/DSL 编译为 CompileOutput。
- CompileOutput 内容：
  - manifests[]：PluginManifest（artifact/io/qos/resource_claims/features/annotations 以及 bindings：timers/bus_subscriptions/bus_publications）。
  - routes[]：Route（from/to/topic/port/content_type/schema_ver/plane/buffer/watermark）。
  - schemas[]：数据面 SchemaDescriptor 去重合集。
  - 详见：`docs/Chronetix/chronetix-flowbridge-api-contract.md`。
  - 插件分类/形态：origin（internal|external）、category（Business|System|Resource）。
- 组件自描述（使用）：`describe-capabilities()`（WIT 导出），可在编译期读取以核对 IO 契约与默认 QoS。

## 3. 编码策略

- 编排产物落盘默认 JSON；控制面在线传输 M2 起推荐 CBOR；通过 content negotiation 兼容。
- WIT ABI 从 JSON 文本演进为 `bytes + content_type`。

## 4. 示例参考

- 侧边车绑定与平台节点的 DSL 片段见 `docs/Chronetix/chronetix-integration-plan.md` §5。

## 5. 变更记录

- 0.1.0（2025-08-22）：初始版本，明确职责与公共接口，落 CompileOutput 扩展字段。

——
相关：
- 集成实现指南：`docs/Chronetix/IMPLEMENTATION_GUIDE.md`
- 稳定入口索引：`docs/Chronetix/INDEX.md`