# Flowbuilder × Chronetix 集成说明

- Status: Draft
- Version: 0.1.0
- Date: 2025-08-17

## 1. 目标（Goals）

- 编排与执行解耦：使用 Flowbuilder 进行 DAG/DSL 编排，Chronetix 负责高精度时间驱动与执行。
- 统一消息边界：以 Envelope（src/dst/topic/corr_id/deadline_ns/...）为稳定契约，控制面走 EventBus、数据面走 DataPort。
- 平滑演进：先单进程（inproc），再 IPC（UDS/SHM），最终可延伸到网络（QUIC），无需改上层图编排。
- 可观测与治理：全链路指标（延迟、jitter、deadline miss、背压溢出）与 Telemetry 事件可回流到 Flowbuilder 面板。

## 2. 方案（Architecture & Mapping）

- 模型映射
  - Flowbuilder 节点/算子 → Chronetix 插件/任务（WASM/dylib/进程隔离均可）；
  - Flowbuilder 边/通道 → EventBus 主题（控制面） + DataPort 通道（数据面）；
  - Flowbuilder 消息 → Envelope（content_type=application/cbor，schema_ver 版本化）。
- 时序与 SLA
  - 窗口/水位线/超时 → deadline_ns + priority；Tick/定时器由 Chronetix TimeManager 产生；
  - 自适应调度（p95/hysteresis）用于在负载波动时稳住 deadline miss。
- 背压
  - sink 侧 credit 通过 DataPort 反馈至上游，WouldBlock 用于快速退避；
  - 可选限速策略插件以阈值/EMA 控制 input rate。
- 安全与隔离
  - 插件形态可选 WASM（WIT ABI 对齐）、dylib 或进程；IPC/网络通道按 RFC_DISTRIBUTED_INTERFACE 定义。

## 3. 接口契约（Contracts）

- Envelope：见 `docs/rfc/draft/RFC_DISTRIBUTED_INTERFACE.md` §2；
- EventBus API：publish/request/subscribe（控制面）；
- DataPort API：acquire_credit/send_frame/on_frame（数据面）；
- Adapter（建议）：
  - FlowAdapter.compile(graph) -> { PluginManifest[], routes(topic/port), schemas }
  - NodeRunner: run(input) -> output（由 Chronetix Executor 托管）。

## 4. 计划（Plan & Milestones）

- M1 PoC（inproc，单进程）
  - 1. Source→Map→Sink 最小 DAG；
  - 2. EventBus 传控制消息，DataPort 传数据帧；
  - 3. 窗口超时映射为 deadline_ns，导出 p95/p99 与 miss 计数；
  - 验收：RTT p95 可观测，背压从 sink 向上游生效，无死锁/泄漏。
- M2 IPC 版（UDS/CBOR + SHM Ring）
  - 1. 将 EventBus 替换为 IpcEventBus（UDS+CBOR）；
  - 2. DataPort 替换为 ShmDataPort（环形共享内存 + 事件通知）；
  - 验收：相同用例下功能等价，p95 延迟退化 ≤ inproc 的 3x。
- M3 网络试点（QUIC + CBOR）
  - 1. 单跳远端算子；
  - 2. 重连/超时/错误分类稳定；
  - 验收：Envelope 编解码兼容，断线恢复正确，指标齐全。

## 5. TODO（Backlog）

- [ ] 在 core/examples 添加 flow_poc.rs（读取一个简化 DAG→ 生成 PluginManifest+路由），跑 inproc 版 PoC。
- [ ] 适配器 crate：chronetix-flowbridge（FlowAdapter/NodeRunner traits + 实现）。
- [ ] IpcEventBus/ShmDataPort 原型（与 RFC 对齐）。
- [ ] 指标桥接：把 Chronetix MetricsRegistry 导入 Flowbuilder 观测面板。
- [ ] Schema registry：保持 content_type/schema_ver 兼容策略与示例。
- [ ] 文档联动：在两个项目 README 互链，并说明部署矩阵（inproc / IPC / 网络）。

## 6. 参考

- Chronetix: `docs/rfc/draft/RFC_DISTRIBUTED_INTERFACE.md`, `docs/rfc/draft/WHITEPAPER_MICROKERNEL.md`
- Flowbuilder: https://github.com/ThneS/flowbuilder
