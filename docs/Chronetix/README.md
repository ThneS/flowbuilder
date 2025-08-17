# Chronetix

高精度时间驱动微内核调度与插件原型。

## 文档入口

- 文档索引: `docs/README.md`
- 微内核白皮书（单机）: `docs/rfc/draft/WHITEPAPER_MICROKERNEL.md`
- 分布式接口 RFC: `docs/rfc/draft/RFC_DISTRIBUTED_INTERFACE.md`
- DataPort RFC: `docs/rfc/draft/RFC_DATAPORT.md`
- 路线图（精简）: `docs/rfc/draft/DEV_PLAN.md`
- 集成（Flowbuilder × Chronetix）: `docs/integration/FLOWBUILDER_INTEGRATION.md`

## 核心聚焦 (Focus)

内核 (Kernel Core)

- 高精度时间调度: Hybrid Timer (多层级轮 + 单调时钟) 实现低漂移、低唤醒抖动
- 事件总线: 轻量 EventBus 支撑定时事件与内部系统广播
- 可扩展执行器: EDF 优先级 + 自适应 tick (Far / Near / Imminent) + 任务生命周期追踪 + 延迟/队列/截断/Deadline miss 指标
- 热更新策略: 资源静默等待 + 强制分离钩子 + 简单 ResourceRegistry（辅助资源可观测与验证）

数据面 (Data Plane)

- 轻量 DataPort：面向高频小帧传输；credit 背压与 WouldBlock 快速路径
- SlabAllocator：内部碎片可视化 (alloc_frag_ratio) + 回收年龄 (alloc_recycle_age_avg_us) + free blocks / largest free block
- 利用率指标：min_credit / peak_utilization_pct / avg/current_util_pct

插件生态 (Plugin Ecosystem)

- 标准生命周期: load → init → ready → start → stop → shutdown / failed
- 失败隔离 & 自动重启: backoff / 最大重试 / 自动卸载策略
- Telemetry Ring: 有界环形缓冲 + 过滤 (include/exclude) + 分类丢弃 (capacity vs filter)
- 统一插件指标: 状态计数、阶段累计耗时与调用次数、失败 & 转换计数

可观测性 (Observability)

- 统一 MetricsRegistry：flatten / with_help / labeled / \*\_into 复用
- HELP 支持 + 自动指标文档生成 + CI diff 防漂移
- Labeling MVP + Cardinality 软/硬阈值监控（gauge 注入 + WARN/ERROR）
- 后续：硬限制 / overflow bucket / histogram 评估 / shard 标签

性能与基准 (Performance & Bench)

- 漂移与执行对比基准 (drift / exec_compare)
- 快照/flatten/exporter 分配 & 时间基线 (track_alloc)
- 预分配 + 缓冲复用 API (`flatten*_into`) 降低高频抓取开销

## 快速开始

```bash
# 运行漂移基准 (快速)
DRIFT_JSON=1 DRIFT_QUICK=1 cargo bench --bench drift -- --quiet
# 执行器指标示例
EXEC_TASKS=200 EXEC_METRICS_JSON=1 cargo run --example exec_metrics --features serde --quiet
# 固定 vs 自适应对比
cargo run --example exec_compare --features serde --quiet
# DataPort 流式帧示例 (可调环境变量见示例文件头注释)
cargo run --example data_port_stream --release --quiet
```

### Try it（一次性命令）

```bash
# QoS 过载演示（全局 runnable 上限 + 丢弃计数 + 队列深度导出）
QOS_GLOBAL_CAP=2 QOS_TASKS=6 cargo run -q -p chronetix-core --example qos_overload_demo

# 生成指标 JSON 清单（含 HELP；需要 serde 特性）
cargo run -q -p chronetix-core --example metrics_inventory_json --features serde > docs/metrics_inventory.json

# 运行 HELP 风格 Lint（缺失或句末有句点会失败）
cargo run -q -p chronetix-core --example metrics_help_lint

# 查看 Hot Reload 指标快照（需启用 hot_reload 特性）
cargo run -q -p chronetix-core --example exec_metrics --features hot_reload,serde | grep '^hotreload\.' || true

# 调整 Cardinality 软阈值并观察（可选）
CHRONETIX_CARDINALITY_SOFT_LIMIT=500 cargo run -q -p chronetix-core --example metrics_inventory_json --features serde >/dev/null
```

> 注：P6（最小版）已完成并收尾，更多指标说明见 `docs/metrics.md` 与最新清单 `docs/metrics_inventory.json`。

## 目录

```
core/    内核 (时间/事件/调度/执行/插件管理/数据面原型)
plugins/ 插件示例
examples/ 演示程序 (exec_metrics / exec_compare / data_port_stream 等)
bench/   基准脚本入口
docs/    文档 (DEV_PLAN / 指南 / RFC / Backlog)
scripts/ CI & Gate 脚本
```

## 当前进度

精简摘要见 `docs/PROGRESS_SUMMARY.md`；完整路线图见 `docs/rfc/draft/DEV_PLAN.md`。DataPort RFC: `docs/rfc/draft/RFC_DATAPORT.md`。
P10 旧实例回收清单模板：`docs/rfc/draft/P10_DEACTIVATE_CHECKLIST.md`。

## 插件与 Telemetry

- FailureRecoveryConfig: auto*unload_failed / restart_enabled / backoff*\_\_\* / max_retries
- Telemetry Ring: lifecycle + meta 事件 (restart_scheduled / restart_loaded / restart_give_up / auto_unload) 支持 include/exclude 过滤
- 使用: `let (pm, ring) = PluginManager::new().with_failure_recovery(cfg).with_telemetry_ring(512, None, None);`

## DataPort 快速说明

| 项         | 说明                                            |
| ---------- | ----------------------------------------------- | ----------- | ------------------------- |
| Capability | `manifest.add_capability("dataport")` (占位)    |
| 分配模型   | SlabAllocator (1MB\* N, bump + free list)       |
| 背压       | credit_high / credit_low (WouldBlock)           |
| API        | send_frame(bytes) -> FrameMeta, recycle(meta)   |
| 回调       | set_callback(                                   | meta, alloc | { ... }) (需显式 recycle) |
| 统计       | port.stats(): frames_sent / bytes_sent / credit |

示例吞吐测试环境变量:

- DATAPORT_FRAMES (默认 10000)
- DATAPORT_FRAME_SIZE (默认 1024)
- DATAPORT_CREDIT (默认 1024)

## 可观测性扩展：热更新与资源卫生

热更新策略（Policy Manager）在切换旧/新实例时提供资源卫生护栏：

- ResourceRegistry（可选）：集中追踪关键资源，支持在 EventBus / DataPort 等处接线自动 upsert/remove，便于回收后检查是否仍有遗留引用。
- 资源静默等待：`with_resource_check()` 注册检查项；结合 `with_resource_quiescent_timeout()` 与 `with_drain_poll_interval()` 控制等待与轮询节奏。
- 强制分离钩子：当超时后仍未完全分离时，通过 `with_force_separate()` 触发自定义“强制分离”流程（例如断开订阅/释放句柄）。
- 质量与可观测性：对应指标以 `hotreload.*` 导出（见下文“指标输出”），便于在 CI/运行时审计资源卫生与回收时延。

进阶：可通过以下环境变量调整 Cardinality 监控与保护：

- `CHRONETIX_CARDINALITY_SOFT_LIMIT`：软阈值（默认 1000），超限 WARN 并导出 gauge。
- `CHRONETIX_CARDINALITY_HARD_LIMIT`：全局硬阈值（可在运行时对特定 family 定制）。
- `CHRONETIX_CARDINALITY_OVERFLOW_BUCKET`：启用溢出桶（实验性）。

## 指标输出 (示例字段)

详见 `docs/metrics.md` 获取完整、HELP 支持与后续规划（含延后标签化设计）。当前：

- 调度: scheduling\_{p50,p95,p99}\_ns
- 执行: execution\_{p50,p95,p99}\_ns
- Offload 队列/调度/GP 执行: offload*queue*_ / offloaded*sched*_ / gp\*exec\*\*
- 任务: deadline\*miss\*\* / panics / total_tasks
- 活跃比: active_ratio_pct(\_f64) / active_ratio_incl_pool_pct(\_f64)
- DataPort: credit / frames_sent / current_util_pct / avg_util_pct(\_f64) / peak_utilization_pct
- 插件: state*counts / failures_total / transitions_total / phase_time*{avg,sum}\_ns

- 热更新: hotreload.switches_total / switches_success / switches_failed / last_switch_duration_ms / active_policy_id
  - 回收阶段资源卫生与强制分离：
    - hotreload.deactivate_leaks
    - hotreload.deactivate_slow_total
    - hotreload.deactivate_verify_failed
    - hotreload.deactivate_resource_check_failed_total
    - hotreload.deactivate_resource_quiescent_timeouts_total
    - hotreload.deactivate_force_separate_invoked_total / hotreload.deactivate_force_separate_failed_total
  - 详见 `docs/metrics.md`

标签化：MVP 已落地（结构 + 导出），尚未为核心指标默认附加 shard 等标签；后续将基于 Cardinality 防护逐步引入。

Prometheus HELP 支持：使用 `flatten_with_help()` + `metrics_to_prometheus_with_help()` 可输出 `# HELP` 行；旧接口仍可用但不含 HELP 行。

## 许可

MIT / Apache-2.0 双许可证，详见 `LICENSE`。

## 贡献

PR / Issue 欢迎。请先阅读 `CONTRIBUTING.md`。

# Chronetix

高精度时间驱动微内核调度与插件原型。

## 核心聚焦 (Focus)

内核 (Kernel Core)

- 高精度时间调度: Hybrid Timer (多层级轮 + 单调时钟) 实现低漂移、低唤醒抖动
- 事件总线: 轻量 EventBus 支撑定时事件与内部系统广播
- 可扩展执行器: EDF 优先级 + 自适应 tick (Far / Near / Imminent) + 任务生命周期追踪 + 延迟/队列/截断/Deadline miss 指标

数据面 (Data Plane)

- 轻量 DataPort：面向高频小帧传输；credit 背压与 WouldBlock 快速路径
- SlabAllocator：内部碎片可视化 (alloc_frag_ratio) + 回收年龄 (alloc_recycle_age_avg_us) + free blocks / largest free block
- 利用率指标：min_credit / peak_utilization_pct / avg/current_util_pct

插件生态 (Plugin Ecosystem)

- 标准生命周期: load → init → ready → start → stop → shutdown / failed
- 失败隔离 & 自动重启: backoff / 最大重试 / 自动卸载策略
- Telemetry Ring: 有界环形缓冲 + 过滤 (include/exclude) + 分类丢弃 (capacity vs filter)
- 统一插件指标: 状态计数、阶段累计耗时与调用次数、失败 & 转换计数

可观测性 (Observability)

- 统一 MetricsRegistry：flatten / with_help / labeled / \*\_into 复用
- HELP 支持 + 自动指标文档生成 + CI diff 防漂移
- Labeling MVP + Cardinality 软阈值监控 (gauge 注入 + WARN)
- 后续：硬限制 / overflow bucket / histogram 评估 / shard 标签

性能与基准 (Performance & Bench)

- 漂移与执行对比基准 (drift / exec_compare)
- 快照/flatten/exporter 分配 & 时间基线 (track_alloc)
- 预分配 + 缓冲复用 API (`flatten*_into`) 降低高频抓取开销

## 快速开始

````bash
# 运行漂移基准 (快速)
 - 热更新策略: 资源静默等待 + 强制分离钩子 + 简单 ResourceRegistry（辅助资源可观测与验证）
# 固定 vs 自适应对比
cargo run --example exec_compare --features serde --quiet
### Try it（一次性命令）

```bash
# QoS 过载演示（全局 runnable 上限 + 丢弃计数 + 队列深度导出）

 # 调整 Cardinality 软阈值并观察（可选）
 CHRONETIX_CARDINALITY_SOFT_LIMIT=500 cargo run -q -p chronetix-core --example metrics_inventory_json --features serde >/dev/null
QOS_GLOBAL_CAP=2 QOS_TASKS=6 cargo run -q -p chronetix-core --example qos_overload_demo

cargo run -q -p chronetix-core --example metrics_inventory_json --features serde > docs/metrics_inventory.json
	- 回收阶段资源卫生与强制分离：
		- hotreload.deactivate_leaks
		- hotreload.deactivate_slow_total
		- hotreload.deactivate_verify_failed
		- hotreload.deactivate_resource_check_failed_total
		- hotreload.deactivate_resource_quiescent_timeouts_total
		- hotreload.deactivate_force_separate_invoked_total / hotreload.deactivate_force_separate_failed_total
	- 详见 `docs/metrics.md`
# 运行 HELP 风格 Lint（缺失或句末有句点会失败）
cargo run -q -p chronetix-core --example metrics_help_lint

# 查看 Hot Reload 指标快照（需启用 hot_reload 特性）

## 目录

examples/ 演示程序 (exec_metrics / exec_compare / data_port_stream 等)
bench/   基准脚本入口
docs/    文档 (DEV_PLAN / 指南 / RFC / Backlog)
scripts/ CI & Gate 脚本
````

## 当前进度

精简摘要见 `docs/PROGRESS_SUMMARY.md`；完整路线图见 `docs/rfc/draft/DEV_PLAN.md`。DataPort RFC: `docs/rfc/draft/RFC_DATAPORT.md`。
P10 旧实例回收清单模板：`docs/rfc/draft/P10_DEACTIVATE_CHECKLIST.md`。

## 插件与 Telemetry

- FailureRecoveryConfig: auto*unload_failed / restart_enabled / backoff*\_\* / max_retries
- Telemetry Ring: lifecycle + meta 事件 (restart_scheduled / restart_loaded / restart_give_up / auto_unload) 支持 include/exclude 过滤
- 使用: `let (pm, ring) = PluginManager::new().with_failure_recovery(cfg).with_telemetry_ring(512, None, None);`

## DataPort 快速说明

| 项         | 说明                                            |
| ---------- | ----------------------------------------------- | ----------- | ------------------------- |
| Capability | `manifest.add_capability("dataport")` (占位)    |
| 分配模型   | SlabAllocator (1MB\* N, bump + free list)       |
| 背压       | credit_high / credit_low (WouldBlock)           |
| API        | send_frame(bytes) -> FrameMeta, recycle(meta)   |
| 回调       | set_callback(                                   | meta, alloc | { ... }) (需显式 recycle) |
| 统计       | port.stats(): frames_sent / bytes_sent / credit |

示例吞吐测试环境变量:

- DATAPORT_FRAMES (默认 10000)
- DATAPORT_FRAME_SIZE (默认 1024)
- DATAPORT_CREDIT (默认 1024)

## 指标输出 (示例字段)

详见 `docs/metrics.md` 获取完整、HELP 支持与后续规划（含延后标签化设计）。当前：

- 调度: scheduling\_{p50,p95,p99}\_ns
- 执行: execution\_{p50,p95,p99}\_ns
- Offload 队列/调度/GP 执行: offload*queue*_ / offloaded*sched*_ / gp*exec*\*
- 任务: deadline*miss*\* / panics / total_tasks
- 活跃比: active_ratio_pct(\_f64) / active_ratio_incl_pool_pct(\_f64)
- DataPort: credit / frames_sent / current_util_pct / avg_util_pct(\_f64) / peak_utilization_pct
- 插件: state*counts / failures_total / transitions_total / phase_time*{avg,sum}\_ns

- 热更新: hotreload.switches_total / switches_success / switches_failed / last_switch_duration_ms / active_policy_id（详见 docs/metrics.md）

标签化：MVP 已落地（结构 + 导出），尚未为核心指标默认附加 shard 等标签；后续将基于 Cardinality 防护逐步引入。

Prometheus HELP 支持：使用 `flatten_with_help()` + `metrics_to_prometheus_with_help()` 可输出 `# HELP` 行；旧接口仍可用但不含 HELP 行。

## 许可

MIT / Apache-2.0 双许可证，详见 `LICENSE`。

## 贡献

PR / Issue 欢迎。请先阅读 `CONTRIBUTING.md`。
