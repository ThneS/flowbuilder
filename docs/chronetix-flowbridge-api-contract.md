# Flowbuilder × Chronetix：Flowbuilder 职责与 FlowAdapter.compile 契约（草案）

- Status: Draft
- Version: 0.1.0
- Date: 2025-08-22
- Applies to: M1 inproc，兼容 M2（IPC）/ M3（网络）
- 参考：docs/chronetix-integration-plan.md（本仓），Chronetix: docs/integration/FLOWBRIDGE_API_CONTRACT.md

本文件面向 Flowbuilder 视角，明确"编排/装配/计划"的职责，并定义唯一需要输出给 Chronetix 的编译期产物接口：FlowAdapter.compile。

---

## 1. 职责（Flowbuilder 侧）

- DAG/DSL 解析与静态分析：节点/边、依赖、条件、并发度校验。
- 组件装配：将"能力组件/资源组件/观测组件"等组合为可部署节点；读取组件自描述（describe-capabilities，可选）。
- 策略固化：schema 兼容性检查；QoS 默认值（deadline/priority/max-inflight/credit）与资源声明；错误前置校验。
- 输出工件：CompileOutput（PluginManifest[]、routes、schemas），可序列化为 JSON（文件/审阅友好）。
- 观测联动：为 Chronetix 指标/遥测预留路由（无需承载存储）。

---

## 2. FlowAdapter.compile（接口）

签名（语言无关）
- 输入：Graph（Flowbuilder 内部模型）
- 输出：CompileOutput（manifests/routes/schemas）
- 失败：InvalidGraph / IncompatibleSchema / MissingArtifact / CyclicDependency 等

输出结构（Shared Types 形状，简版）

- Envelope（控制面边界）
  ```json
  { "src":"node-A","dst":"node-B","topic":"flow/edge/A-B","corr_id":"c-123","deadline_ns":1680000000000,"priority":5,"content_type":"application/arrow-ipc","schema_ver":"v1","ext":{} }
  ```
- SchemaDescriptor
  ```json
  { "content_type":"application/arrow-ipc","schema_ver":"v1","schema_ref":"registry://schemas/user_events@v1" }
  ```
- PluginManifest（每节点）
  ```json
  { "plugin_id":"map-transform-wasm","version":"0.1.0","category":"Capability","role":"Transform","artifact":{"kind":"Wasm","uri":"file://plugins/map.wasm"},"io":{"inputs":[{"content_type":"application/arrow-ipc","schema_ver":"v1"}],"outputs":[{"content_type":"application/arrow-ipc","schema_ver":"v1"}],"batch_hints":{"preferred_rows":10000,"preferred_bytes":2097152}},"qos":{"deadline_ns":2000000,"priority":5,"retry":3,"max_inflight":32,"credit_high":64,"credit_low":16},"resource_claims":{"cpu_millis":200,"memory_bytes":134217728},"features":{"inproc":true},"annotations":{} }
  ```
- Route（边/通道）
  ```json
  { "from":"node-A","to":"node-C","topic":"flow/edge/A-C","port":"dataport/A-C","content_type":"application/arrow-ipc","schema_ver":"v1","buffer":64,"watermark":48 }
  ```
- CompileOutput
  ```json
  { "manifests":[],"routes":[],"schemas":[] }
  ```

约束与校验
- 边必须指向存在的节点；多入/多出需逐边 schema 对齐。
- content_type/schema_ver 在 from→to 之间必须兼容。
- QoS 与资源声明若未显式指定，应按 Planner 的默认与继承规则补全。
- 输出必须可稳定序列化为 JSON；线上传输编码可与宿主协商（建议 M2 起 CBOR）。

---

## 3. 组件自描述（可选）

- 接口：describe-capabilities() → JSON（由组件导出，详见 Chronetix 文档 §7）
- 用途：在编译期填充/校验 IO 契约（inputs/outputs）、默认 batch_hints/QoS、hooks 列表。

---

## 4. 生命周期（编译→部署）

1) Compile：读取 YAML/DSL → 静态分析 → 校验 schema/QoS → 生成 CompileOutput。
2) Deploy（Chronetix）：读取 CompileOutput → 执行器加载 NodeRunner → 绑定 EventBus/DataPort。
3) Run：on-frame/on-tick 驱动；WouldBlock 背压退避；指标/遥测回流。

---

## 5. 兼容性与编码

- CompileOutput 文件：默认 application/json（便于审阅）。
- EventBus/控制面：M2 起建议 application/cbor（更紧凑）；由宿主协商。
- 字段演进：字段可增不减；未知字段忽略但透传。

---

## 6. 参考与后续

- Flowbuilder 计划：docs/chronetix-integration-plan.md
- Chronetix API 合同：Chronetix/docs/integration/FLOWBRIDGE_API_CONTRACT.md
- M1 后续：在 examples 中提供最小 DAG → CompileOutput 示例与快测脚本。