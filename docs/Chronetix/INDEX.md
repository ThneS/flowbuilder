# Flowbuilder × Chronetix 文档索引（稳定入口）

- 状态：Draft
- 日期：2025-08-22

本目录聚合 Chronetix 集成相关文档的稳定链接，便于跨仓引用与审阅。

- 职责与公共接口（Flowbuilder 侧）：docs/Chronetix/RESPONSIBILITIES_AND_APIS.md
- 集成计划（Envelope/EventBus/DataPort 对齐）：docs/Chronetix/chronetix-integration-plan.md
- FlowAdapter.compile 契约（编译期产物接口）：docs/Chronetix/chronetix-flowbridge-api-contract.md
- 集成实现指南（详细版）：docs/Chronetix/IMPLEMENTATION_GUIDE.md

补充：
- Chronetix 侧总览：docs/Chronetix/README.md
- WIT 接口草案：wit/flow/*.wit（common/bus/stream/blob/metrics）

示例：
- 最小 DAG（YAML，含 origin/category）：docs/Chronetix/examples/minimal_dag.yaml
- 对应 CompileOutput（JSON）：docs/Chronetix/examples/minimal_compile_output.json

备注：所有输出产物默认 JSON（便于审阅）；控制面传输建议 M2 起使用 CBOR，由宿主协商。
