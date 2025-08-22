# Flowbridge（Flowbuilder 侧）职责与公共接口（草案）

- Status: Draft
- Version: 0.1.0
- Date: 2025-08-22

本文件聚焦于 Flowbuilder 在与 Chronetix 集成中的职责与公共接口（契约），对应 Chronetix 仓的《docs/integration/FLOWBRIDGE_API_CONTRACT.md》。

## 1. 范围（Scope）

- 编排/装配/计划（Plan）：Flowbuilder 解析 YAML/DSL，做静态分析与校验，生成运行期产物。
- 非范围：运行/调度/传输/背压执行由 Chronetix 托管（EventBus、DataPort、Executor）。

## 2. 职责（Responsibilities）

- 能力发现与兼容性
  - 读取 WASM 组件的 describe-capabilities（WIT 导出），形成 IO 契约与默认 QoS/batch hints。
  - 校验节点依赖、schema 兼容（content_type/schema_ver），并在边上固化。
- 编译输出（CompileOutput）
  - 生成 PluginManifest[]（每个节点）、routes[]（每条边）、schemas[]（去重合集）。
  - 固化 QoS 默认（deadline_ns/priority/retry/max_inflight/credit 阈值等）与资源声明（cpu/mem/fds 等）。
  - 输出应为 JSON 文件/对象，便于审阅与版本化。
- 规划与映射
  - 节点/算子 → 插件/任务（WASM/dylib/进程）；
  - 边/通道 → EventBus 主题 + DataPort 端口；
  - 时间窗口/水位线/超时 → deadline_ns + priority 默认值。
- 观测联动
  - 将 Chronetix 指标与遥测在 UI 中展现；不承载度量存储与采集实现。

## 3. 公共接口（Public API）

### 3.1 FlowAdapter.compile

- 入口：`FlowAdapter.compile(graph) -> { manifests, routes, schemas }`
- 输入：Graph（内部模型即可，外部不强制形状）
- 输出：CompileOutput（JSON 可序列化）
- 失败：结构化错误（InvalidGraph/IncompatibleSchema/MissingArtifact 等）

JSON 形状（示例）：

```json
{
  "manifests": [
    {
      "plugin_id": "source-wasm",
      "version": "0.1.0",
      "category": "Capability",
      "role": "Source",
      "artifact": {"kind": "Wasm", "uri": "file://plugins/source.wasm"},
      "io": {
        "inputs": [],
        "outputs": [{"content_type": "application/arrow-ipc", "schema_ver": "v1"}],
        "batch_hints": {"preferred_rows": 10000, "preferred_bytes": 1048576}
      },
      "qos": {"deadline_ns": 2000000, "priority": 5},
      "resource_claims": {"cpu_millis": 200, "memory_bytes": 134217728}
    }
  ],
  "routes": [
    {
      "from": "node-A",
      "to": "node-B",
      "topic": "flow/edge/A-B",
      "port": "dataport/A-B",
      "content_type": "application/arrow-ipc",
      "schema_ver": "v1",
      "buffer": 64,
      "watermark": 48
    }
  ],
  "schemas": [
    {"content_type": "application/arrow-ipc", "schema_ver": "v1", "schema_ref": "registry://schemas/demo@v1"}
  ]
}
```

### 3.2 编排期如何使用 describe-capabilities

- 可选阶段：在编译期调用组件导出 `describe-capabilities()`（WIT），获取组件声明的 inputs/outputs/hooks/batch_hints/qos/features。
- 优先级：显式 DSL/YAML 配置 > 组件自描述默认值。

## 4. Schema 与 QoS 策略

- Schema：所有边需声明 content_type/schema_ver；发生不兼容需在编译期报错或插入编解码算子。
- QoS：默认值由 Planner 计算并固化；deadline_ns/priority 会传入 NodeRunner 以驱动调度。

## 5. 生命周期（编排→运行）

1) Flowbuilder：解析/校验 → 调用 describe-capabilities（可选）→ 生成 CompileOutput（JSON）。
2) Chronetix：读取 CompileOutput → 启动 NodeRunner → 绑定 EventBus/DataPort → 执行。

## 6. 版本与演进

- CompileOutput 字段允许"可增不减"。
- 编码：文件/接口输出默认 JSON；在运行传输中可由 Chronetix 侧转为 CBOR。

## 7. 参考链接

- Chronetix：`docs/integration/FLOWBRIDGE_API_CONTRACT.md`（接口总览）
- Chronetix：`docs/integration/ENCODING_OPTIONS.md`（编码选项）
- Flowbuilder：`docs/chronetix-integration-plan.md`