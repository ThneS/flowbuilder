# Flowbuilder × Chronetix 集成实现指南（详细版）

- Status: Draft
- Version: 0.2.0
- Date: 2025-08-22

## 0. 摘要（TL;DR）

- 编排（Flowbuilder）：从 DSL/YAML 编译为 PluginManifest[] + routes + schemas（Envelope 契约），不关心执行细节。详见 `docs/Chronetix/chronetix-flowbridge-api-contract.md`。
- 执行（Chronetix）：提供 EventBus（控制面）与 DataPort（数据面），落实背压（credit/WouldBlock）、SLA（deadline/priority）、可观测与热更新。
- 组件化（WIT）：在 Wasm 形态时，通过统一的 WIT 接口承载事件总线与数据通道（含 Arrow/Blob/Streams）。接口草案位于本仓 `wit/flow/*.wit`。

## 1. 数据域与类型映射

- 域划分
  - Intra-Module：模块内部强类型数据，不经总线。
  - Inter-Module：跨模块数据，走统一 Envelope + Bus/Stream/Blob。

- 数据类型
  - 控制/标记/小结构体：CBOR/MsgPack，通过 EventBus（publish/request/subscribe）。
  - 业务 bytes（中等/大）：通过 Stream（wasi:io/streams），背压由 DataPort 承担。
  - 分析/批量：Arrow IPC Streaming（在 Stream 上承载）。
  - 大对象（Blob）：走内容寻址引用 + 流式上传/下载（可用 presign URL）。
  - 日志/度量/追踪：结构化事件 + MetricsRegistry 桥接至 Flowbuilder 面板。
  - 资源配置（Resource）：建议 JSON/CBOR，控制面传输，注册 schema（schemas[]）。

- 场景
  - 通知 / RPC / PubSub / 广播：EventBus。
  - 流式拉/推 / Fan-in/out / Scatter-Gather：Stream（DataPort credit）+ EventBus 控制。
  - 回放/补数：持久订阅 + WAL（后续分布式阶段）。
  - 时间与水位：Chronetix TimeManager 产生 Tick/Deadline，Envelope 携带 deadline_ns/priority。

## 2. 编排期（Flowbuilder） → 运行期（Chronetix）映射

- FlowAdapter.compile(graph) 产物：
  - PluginManifest[]：节点/算子描述（WASM/dylib/process）。
  - routes：EventBus topics + Stream/DataPort 端口拓扑。
  - schemas：content_type（application/cbor/arrow+ipc/...）与 schema_ver。
  - defaults：deadline_ns/priority 继承规则与 QoS 的默认。

插件形态与分类（建议实现规范）：
- origin：internal | external
  - internal：来自 Chronetix 内部组件（Wasm Component 形态），如时间/总线/统计等系统能力；Manifest 与业务插件一致。
  - external：业务或第三方交付的插件。
- category：Business | System | Resource
  - Business：具体业务逻辑（例如 HTTP 收发、协议编解码、业务 ETL）。
  - System：平台系统能力（定时器、事件总线、指标统计等）。
  - Resource：环境与资源信息提供者（IP/掩码/默认网关、端口、媒体位置等），通常在控制面输出配置数据。

Manifest 建议：
- artifact.kind 使用 "WasmComponent" 表达组件模型；origin 填写 internal/external。
- Resource 类输出 content_type 建议 application/json 或 application/cbor，提供 schema_ref 并入 schemas[] 去重集合。

- NodeRunner（Chronetix 托管）：
  - run(input) -> output：由 Chronetix Executor 执行；TimeManager 提供 tick/deadline；DataPort 提供 credit 背压；WouldBlock 快速退避。
  - 安全与隔离：WASM（WIT ABI）、dylib 或进程；IPC/网络（UDS/SHM/QUIC）。

## 3. 接口契约（WIT + Traits）

- WIT（见 wit/flow/*.wit）：
  - flow:common：qos-class、trace、headers、envelope 等通用类型。
  - flow:bus：event-bus（小消息/控制面）。
  - flow:stream：channel（创建/获取 tx/rx，背压由宿主流实现）。
  - flow:blob：大对象上传/下载/预签名 URL。
  - flow:metrics：轻量拉取/后续可扩 Streams。

- Traits（见 crates/chronetix-flowbridge）：
  - FlowAdapter：从 Flowbuilder DAG/DSL 生成 manifests/routes/schemas。
  - NodeRunner：Chronetix 运行入口的抽象封装（inproc 胶水，后续可 IPC/QUIC）。

参考：
- 集成计划：`docs/Chronetix/chronetix-integration-plan.md`
- 职责与公共接口：`docs/Chronetix/RESPONSIBILITIES_AND_APIS.md`

## 4. 背压、SLA 与 QoS

- 背压（数据面）：
  - DataPort credit → 上游生产者限速；WouldBlock 快速退避；高/低水位线触发策略。
  - 流式接口：wasi streams（incoming/outgoing）+ pollable 可读/可写，避免 busy-poll。

- SLA（控制面）：
  - Envelope.deadline_ns / priority：TimeManager/Executor 全链路跟踪 deadline miss。
  - 自适应：p95/hysteresis 策略在负载波动时稳定 SLA。

- QoS/优先级：
  - topic/channel 维度设置 best-effort/normal/high/realtime；配额与公平调度。

## 5. 可观测性与治理

- 统一指标：Chronetix MetricsRegistry ←→ Flowbuilder UI（桥接）。
- 追踪：W3C trace-context 注入/透传（Envelope/headers）。
- Schema：Schema registry（schema-id / schema-ver）与兼容策略；灰度升级。
- 数据治理：TTL/保留策略、敏感字段脱敏、审计、血缘（后续）。

## 6. 部署矩阵（演进）

1) Inproc（单进程 PoC）：最小成本落地，功能等价。
2) IPC（UDS+CBOR + SHM Ring）：低延时可控，背压外显。
3) 网络（QUIC+CBOR）：远端算子单跳，断线重连与错误分类完善。

## 7. 里程碑（与文档对齐）

- M1 Inproc PoC：Source→Map→Sink，EventBus 控制、Stream 数据；deadline 映射；p95/p99 与 miss 计数。
- M2 IPC：IpcEventBus（UDS+CBOR）、ShmDataPort（环形共享内存）。
- M3 网络：QUIC + CBOR，断线重连/超时/错误分类，指标齐全。

## 8. 风险与开放问题

- Envelope/schema 版本锁定与升级策略。
- 重试/超时归属：建议 Chronetix 托管（Flowbuilder 描述策略）。
- 并发与配额：跨进程限流/配额由宿主协调。