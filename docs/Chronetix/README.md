# Chronetix

高精度时间驱动微内核调度与插件原型（文档索引与集成指引入口）。

## 文档入口

- 文档索引（本仓）: docs/README.md
- 分布式接口 RFC: docs/rfc/draft/RFC_DISTRIBUTED_INTERFACE.md
- DataPort RFC: docs/rfc/draft/RFC_DATAPORT.md
- 路线图（精简）: docs/rfc/draft/DEV_PLAN.md
- Flowbuilder × Chronetix 集成：docs/integration/FLOWBUILDER_INTEGRATION.md

## 核心聚焦（摘要）

- 高精度时间调度（Hybrid Timer）与事件总线
- 可扩展执行器（EDF + 自适应 tick）与任务生命周期观测
- 数据面 DataPort（credit 背压、WouldBlock 快速路径）
- 插件生态（WASM/dylib/process）与热更新策略
- 可观测性（MetricsRegistry + HELP）

## 快速开始

```bash
# 漂移基准（快速）
DRIFT_JSON=1 DRIFT_QUICK=1 cargo bench --bench drift -- --quiet
# 执行器指标示例
EXEC_TASKS=200 EXEC_METRICS_JSON=1 cargo run --example exec_metrics --features serde --quiet
# 固定 vs 自适应对比
cargo run --example exec_compare --features serde --quiet
# DataPort 流式帧示例（可调环境变量见示例文件头注释）
cargo run --example data_port_stream --release --quiet
```

更多一次性命令与可观测性示例见项目 README 与 docs。

## 目录（示例）

```
core/     内核 (时间/事件/调度/执行/插件管理/数据面原型)
plugins/  插件示例
examples/ 演示程序 (exec_metrics / exec_compare / data_port_stream 等)
bench/    基准脚本入口
docs/     文档 (DEV_PLAN / 指南 / RFC / Backlog)
scripts/  CI & Gate 脚本
```

## 与 Flowbuilder 的集成

- 职责与公共接口：docs/Chronetix/RESPONSIBILITIES_AND_APIS.md
- 集成计划：docs/Chronetix/chronetix-integration-plan.md
- 编译期接口契约：docs/Chronetix/chronetix-flowbridge-api-contract.md
- 实现指南（详细版）：docs/Chronetix/IMPLEMENTATION_GUIDE.md

以上文档路径均已对齐为 docs/Chronetix 下的稳定入口，便于跨仓链接与审阅。

示例：
- 最小 DAG（YAML，含 origin/category）：docs/Chronetix/examples/minimal_dag.yaml
- 对应 CompileOutput（JSON）：docs/Chronetix/examples/minimal_compile_output.json
