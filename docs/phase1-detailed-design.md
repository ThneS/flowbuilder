# FlowBuilder 第一阶段详细设计文档

## 🚀 Phase 0: 单人+AI 快速原型 (0-30 天)

### 超级个人开发模式

**目标**: 利用 AI 协作，一人在 30 天内产出可演示的工作流引擎原型

#### AI 驱动的开发策略

##### 1. AI 工具栈集成

```
核心AI工具:
├── GitHub Copilot       # 实时代码生成 (80%代码AI生成)
├── Cursor IDE          # AI原生开发环境
├── Claude/ChatGPT      # 架构设计顾问
├── v0.dev              # UI组件快速生成
└── GitHub Actions      # 自动化CI/CD

开发加速器:
├── Rust Analyzer       # 智能补全和重构
├── Tauri Studio        # 桌面应用脚手架
├── Prisma             # 数据库Schema生成
└── OpenAPI Generator   # API客户端自动生成
```

##### 2. 极简技术栈

```rust
// 技术栈选择：专注核心，避免过度工程
Backend:    Rust + Tokio + Axum + SQLite
Frontend:   React + TypeScript + Tailwind
Desktop:    Tauri (Rust + WebView)
Deployment: Docker + Railway/Fly.io
```

#### 30 天开发时间线

##### Week 1: 核心引擎 (Days 1-7)

```rust
// AI生成的核心数据结构
pub struct WorkflowEngine {
    parser: YamlParser,
    executor: TaskExecutor,
    scheduler: SimpleScheduler,
    storage: SqliteStorage,
}

// 基础功能清单
- [x] YAML工作流解析器
- [x] 任务依赖图构建
- [x] 基础执行器 (script, http, shell)
- [x] SQLite数据持久化
- [x] 简单的CLI工具
```

##### Week 2: Web 界面 (Days 8-14)

```typescript
// v0.dev生成的React组件
- [x] 工作流列表页面
- [x] 可视化流程图 (ReactFlow)
- [x] 执行监控面板
- [x] 任务日志查看器
- [x] REST API (Axum)
```

##### Week 3: 桌面应用 (Days 15-21)

```rust
// Tauri桌面应用集成
- [x] 跨平台桌面应用
- [x] 本地文件系统访问
- [x] 系统托盘集成
- [x] 实时状态通知
- [x] 性能监控面板
```

##### Week 4: 完善发布 (Days 22-30)

-   [x] Docker 容器化
-   [x] 云平台部署
-   [x] 自动化测试
-   [x] 用户文档
-   [x] 演示视频

#### AI 协作最佳实践

##### 高效提示词模板

**架构设计提示**:

```
作为高级Rust开发者，设计一个高性能工作流引擎的[具体模块]：

需求：
1. 支持YAML配置解析
2. 异步任务并发执行
3. 内存安全和错误处理
4. 可扩展的插件架构

请提供：
- 完整的数据结构定义
- trait接口设计
- 核心实现逻辑
- 使用示例和测试
```

**代码生成提示**:

```
实现一个Rust工作流执行器，要求：
- 使用Tokio异步运行时
- 支持任务依赖和并发
- 包含错误重试机制
- 提供进度回调接口

请生成完整代码，包含详细注释。
```

##### AI 开发工作流

1. **设计阶段**: 与 Claude 讨论架构设计
2. **编码阶段**: Copilot 实时代码生成
3. **调试阶段**: AI 辅助错误分析
4. **优化阶段**: AI 建议性能改进
5. **文档阶段**: AI 生成 API 文档

#### 原型功能规格

##### 核心功能

```yaml
# 支持的工作流示例
name: "data-pipeline"
version: "1.0"
description: "数据处理流水线"

tasks:
    - name: "fetch_data"
      type: "http"
      config:
          url: "https://api.example.com/data"
          method: "GET"
          timeout: 30

    - name: "process_data"
      type: "script"
      depends_on: ["fetch_data"]
      config:
          language: "python"
          code: |
              import json
              data = json.loads(input_data)
              result = {"count": len(data), "processed_at": datetime.now()}
              print(json.dumps(result))

    - name: "send_notification"
      type: "shell"
      depends_on: ["process_data"]
      config:
          command: 'curl -X POST https://hooks.slack.com/webhook -d ''{"text": "Pipeline completed"}}'''
```

##### 界面功能

-   **工作流设计器**: 拖拽式可视化编辑
-   **实时监控**: 任务执行状态和日志
-   **性能面板**: CPU、内存、执行时间统计
-   **插件管理**: 任务类型扩展和配置

#### 性能目标 (原型阶段)

| 指标           | 目标值  | 验证方式 |
| -------------- | ------- | -------- |
| 工作流启动延迟 | < 100ms | 单元测试 |
| 任务并发数     | >= 50   | 压力测试 |
| 内存使用       | < 50MB  | 监控面板 |
| UI 响应时间    | < 200ms | 手动测试 |
| 包大小         | < 20MB  | 构建产物 |

#### 部署策略

##### 本地开发

```bash
# 一键启动开发环境
just dev         # 启动后端 + 前端
just build       # 构建所有组件
just test        # 运行测试套件
just deploy      # 部署到云平台
```

##### 云平台部署

```dockerfile
# 优化的Docker镜像
FROM rust:1.75-alpine as builder
# ... 构建步骤 (AI生成)

FROM alpine:latest
# ... 运行时环境 (AI生成)
```

#### 风险控制

##### 技术风险

-   **AI 依赖**: 核心逻辑人工 review，AI 辅助非关键代码
-   **性能债务**: 持续 profiling，及时优化瓶颈
-   **架构复杂性**: 保持模块化，避免过早优化

##### 项目风险

-   **功能蔓延**: 严格控制 MVP 范围
-   **质量问题**: AI 生成代码必须经过测试
-   **时间压力**: 每周迭代，及时调整优先级

这个快速原型阶段为后续的完整开发奠定基础，验证技术可行性和用户需求。

---

## 极限性能优化策略

### 1. 零拷贝数据传输

**目标**: 消除不必要的内存拷贝，提升数据传输效率

**技术实现**:

-   **Apache Arrow 内存布局**: 列式内存格式，支持零拷贝切片
-   **内存映射文件**: 利用 mmap 实现大文件零拷贝读取
-   **共享内存**: 进程间通信避免数据拷贝
-   **Ring Buffer**: 高性能环形缓冲区

```rust
// 零拷贝数据传输示例
pub struct ZeroCopyChannel {
    shared_buffer: Arc<SharedMemory>,
    ring_buffer: LockFreeRingBuffer,
}

impl ZeroCopyChannel {
    pub fn send_zero_copy(&self, data: &ArrowArray) -> Result<()> {
        // 直接引用，避免拷贝
        let slice = data.as_slice();
        self.ring_buffer.push_ref(slice)?;
        Ok(())
    }
}
```

### 2. SIMD 指令优化

**目标**: 利用 CPU 向量指令集加速并行计算

**应用场景**:

-   批量数据处理和转换
-   向量计算和相似度搜索
-   压缩和解压缩算法
-   加密和哈希计算

```rust
// SIMD 优化示例
use std::arch::x86_64::*;

pub fn simd_sum(data: &[f32]) -> f32 {
    unsafe {
        let mut sum = _mm256_setzero_ps();
        for chunk in data.chunks_exact(8) {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            sum = _mm256_add_ps(sum, vec);
        }
        // 水平求和
        _mm256_hadd_ps(sum, sum)
    }
}
```

### 3. 无锁数据结构

**目标**: 减少锁竞争，提高并发性能

**核心组件**:

-   **Lock-Free Queue**: 无锁队列用于任务传递
-   **Atomic Operations**: 原子操作实现状态管理
-   **Hazard Pointers**: 内存安全的无锁算法
-   **Read-Copy-Update (RCU)**: 读多写少场景优化

```rust
// 无锁队列实现
use crossbeam::queue::SegQueue;

pub struct LockFreeTaskQueue {
    queue: SegQueue<Task>,
    stats: AtomicU64,
}

impl LockFreeTaskQueue {
    pub fn push(&self, task: Task) {
        self.queue.push(task);
        self.stats.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pop(&self) -> Option<Task> {
        self.queue.pop()
    }
}
```

### 4. 智能内存管理

**目标**: 减少内存分配开销，提高内存利用率

**策略**:

-   **对象池模式**: 预分配对象，避免频繁分配释放
-   **Arena 分配器**: 批量分配内存，减少碎片
-   **内存池**: 分级内存池管理不同大小对象
-   **压缩 GC**: 智能垃圾回收和内存整理

```rust
// Arena 分配器示例
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current: AtomicUsize,
    offset: AtomicUsize,
}

impl Arena {
    pub fn alloc<T>(&self, value: T) -> &T {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        unsafe {
            ptr.cast::<T>().write(value);
            &*ptr.cast::<T>()
        }
    }
}
```

## 智能调度系统

### 1. 工作窃取算法

**目标**: 实现动态负载均衡，最大化 CPU 利用率

```rust
pub struct WorkStealingScheduler {
    local_queues: Vec<LocalQueue>,
    global_queue: GlobalQueue,
    workers: Vec<Worker>,
}

impl WorkStealingScheduler {
    pub fn schedule(&self, task: Task) {
        // 优先本地队列
        if let Some(local) = self.current_local_queue() {
            if local.try_push(task) {
                return;
            }
        }
        // 回退到全局队列
        self.global_queue.push(task);
    }

    pub fn steal_work(&self, worker_id: usize) -> Option<Task> {
        // 随机选择其他 worker 窃取任务
        let target = self.random_worker_except(worker_id);
        self.local_queues[target].steal()
    }
}
```

### 2. NUMA 感知调度

**目标**: 优化 NUMA 架构下的内存访问性能

```rust
pub struct NumaAwareScheduler {
    numa_nodes: Vec<NumaNode>,
    task_affinity: HashMap<TaskId, NumaNodeId>,
}

impl NumaAwareScheduler {
    pub fn schedule_with_affinity(&self, task: Task) {
        let preferred_node = self.get_preferred_numa_node(&task);
        let worker = self.find_best_worker(preferred_node);
        worker.schedule(task);
    }

    fn get_preferred_numa_node(&self, task: &Task) -> NumaNodeId {
        // 基于数据局部性选择 NUMA 节点
        task.input_data.primary_memory_location()
    }
}
```

## 高效存储引擎

### 1. 列式存储优化

**目标**: 优化分析型工作负载的存储和查询性能

```rust
pub struct ColumnStore {
    columns: HashMap<String, Column>,
    row_groups: Vec<RowGroup>,
    compression: CompressionScheme,
}

impl ColumnStore {
    pub fn insert_batch(&mut self, batch: &RecordBatch) -> Result<()> {
        for (i, array) in batch.columns().iter().enumerate() {
            let column_name = batch.schema().field(i).name();
            self.columns.get_mut(column_name)
                .unwrap()
                .append_array(array)?;
        }
        Ok(())
    }

    pub fn query_column(&self, column: &str, filter: &Filter) -> Result<Array> {
        let column_data = self.columns.get(column).unwrap();
        column_data.filter(filter)
    }
}
```

### 2. 自适应索引

**目标**: 根据查询模式自动创建和维护索引

```rust
pub struct AdaptiveIndexManager {
    indexes: HashMap<String, Index>,
    query_stats: QueryStatistics,
    index_advisor: IndexAdvisor,
}

impl AdaptiveIndexManager {
    pub fn suggest_indexes(&self) -> Vec<IndexRecommendation> {
        let frequent_queries = self.query_stats.get_frequent_patterns();
        self.index_advisor.analyze(frequent_queries)
    }

    pub fn auto_create_index(&mut self, recommendation: IndexRecommendation) {
        if self.should_create_index(&recommendation) {
            let index = self.build_index(recommendation);
            self.indexes.insert(recommendation.name, index);
        }
    }
}
```

## 第一阶段里程碑规划

### M1: 核心架构重构 (0-3 个月)

**目标**: 建立七层架构基础，实现核心组件

**关键任务**:

-   [ ] 设计和实现七层架构接口
-   [ ] 重构现有代码到新架构
-   [ ] 实现插件系统框架
-   [ ] 建立性能基准测试

**验收标准**:

-   所有层次接口定义完成
-   核心功能在新架构下正常运行
-   插件系统支持热加载
-   性能基准测试套件可运行

### M2: 性能优化引擎 (3-6 个月)

**目标**: 实现极限性能优化，达到微秒级延迟

**关键任务**:

-   [ ] 实现零拷贝数据传输
-   [ ] 集成 SIMD 指令优化
-   [ ] 构建无锁数据结构
-   [ ] 优化内存分配策略

**验收标准**:

-   执行延迟 P99 < 1ms, P50 < 100μs
-   内存拷贝次数减少 90%
-   并发性能提升 10x
-   CPU 利用率 > 90%

### M3: 智能调度系统 (6-9 个月)

**目标**: 构建高效的任务调度和资源管理

**关键任务**:

-   [ ] 实现工作窃取调度器
-   [ ] 添加 NUMA 感知优化
-   [ ] 构建自适应负载均衡
-   [ ] 实现智能资源分配

**验收标准**:

-   支持 10K+ 并发工作流
-   负载均衡效率 > 95%
-   资源利用率自动优化
-   调度延迟 < 10μs

### M4: 企业级特性 (9-12 个月)

**目标**: 添加企业级功能，为分布式演进做准备

**关键任务**:

-   [ ] 实现多租户架构
-   [ ] 添加 RBAC 权限系统
-   [ ] 构建审计日志系统
-   [ ] 完善监控和可观测性

**验收标准**:

-   支持 1000+ 租户
-   权限检查延迟 < 1μs
-   审计日志覆盖率 100%
-   监控指标 > 100 项

## 性能目标与基准

### 核心性能指标

| 指标           | 当前值 | 目标值  | 提升倍数 |
| -------------- | ------ | ------- | -------- |
| 执行延迟 (P50) | ~10ms  | <100μs  | 100x     |
| 执行延迟 (P99) | ~50ms  | <1ms    | 50x      |
| 吞吐量         | ~1K/s  | >100K/s | 100x     |
| 内存效率       | ~70%   | >90%    | 1.3x     |
| CPU 利用率     | ~60%   | >90%    | 1.5x     |
| 并发工作流     | ~100   | >10K    | 100x     |

### 基准测试场景

1. **微基准测试**: 单个操作的极限性能
2. **负载测试**: 高并发场景下的稳定性
3. **压力测试**: 资源极限下的表现
4. **持久化测试**: 长期运行的稳定性

### 竞品对比

| 平台            | 延迟       | 吞吐量      | 并发数   | 特色         |
| --------------- | ---------- | ----------- | -------- | ------------ |
| Airflow         | ~1s        | ~1K/s       | ~1K      | 成熟生态     |
| Temporal        | ~100ms     | ~10K/s      | ~5K      | 持久化       |
| **FlowBuilder** | **<100μs** | **>100K/s** | **>10K** | **极限性能** |

## 技术栈选择

### 第一阶段技术栈

```
核心语言: Rust (高性能、内存安全)
数据处理: Apache Arrow + Polars
存储引擎: RocksDB + Redis
网络通信: Tokio + Async-std
AI/ML: Candle + Ort (ONNX Runtime)
监控追踪: OpenTelemetry + Jaeger
前端界面: TypeScript + React + Tauri
```

## 模块化架构设计

### 核心模块划分

基于关注点分离和独立演进的原则，FlowBuilder 将拆分为以下专门化模块：

#### 1. flowbuilder-core (核心引擎)

**职责**: 工作流执行的核心逻辑和算法

-   工作流解析和编译
-   执行计划生成和优化
-   数据流管理和状态追踪
-   插件系统框架

```rust
// flowbuilder-core 核心接口
pub trait FlowEngine {
    fn compile_workflow(&self, definition: &WorkflowDef) -> Result<ExecutionPlan>;
    fn execute_plan(&self, plan: ExecutionPlan) -> Result<ExecutionResult>;
    fn get_execution_state(&self, flow_id: &FlowId) -> Result<FlowState>;
}

pub struct CoreEngine {
    compiler: WorkflowCompiler,
    optimizer: ExecutionOptimizer,
    state_manager: StateManager,
    plugin_registry: PluginRegistry,
}
```

#### 2. flowrunner (运行时引擎)

**职责**: 高性能的工作流运行时环境

-   任务调度和执行
-   资源管理和分配
-   性能监控和优化
-   故障恢复和重试

```rust
// flowrunner 运行时接口
pub trait RuntimeEngine {
    fn spawn_flow(&self, plan: ExecutionPlan) -> Result<FlowHandle>;
    fn manage_resources(&self) -> Result<ResourceUsage>;
    fn handle_failure(&self, error: ExecutionError) -> Result<RecoveryAction>;
}

pub struct HighPerformanceRunner {
    scheduler: WorkStealingScheduler,
    resource_manager: NumaAwareResourceManager,
    monitor: RealTimeMonitor,
    recovery_engine: FaultToleranceEngine,
}
```

#### 3. flowui (用户界面)

**职责**: 可视化工作流设计和管理界面

-   拖拽式工作流设计器
-   实时执行监控面板
-   配置管理界面
-   用户权限管理

```typescript
// flowui 核心组件
interface FlowDesigner {
    createWorkflow(): WorkflowBuilder;
    editWorkflow(id: string): WorkflowEditor;
    validateWorkflow(definition: WorkflowDef): ValidationResult;
}

interface ExecutionDashboard {
    monitorExecution(flowId: string): ExecutionMonitor;
    showMetrics(): MetricsPanel;
    manageResources(): ResourcePanel;
}
```

#### 4. flowbuilder-storage (存储引擎)

**职责**: 高效的数据存储和检索

-   工作流定义存储
-   执行状态持久化
-   结果数据管理
-   元数据索引

```rust
// flowbuilder-storage 存储接口
pub trait StorageEngine {
    fn store_workflow(&self, workflow: &WorkflowDef) -> Result<WorkflowId>;
    fn load_workflow(&self, id: &WorkflowId) -> Result<WorkflowDef>;
    fn persist_state(&self, state: &ExecutionState) -> Result<()>;
    fn query_history(&self, query: &HistoryQuery) -> Result<Vec<ExecutionRecord>>;
}

pub struct HybridStorage {
    metadata_store: PostgresStore,
    state_store: RedisStore,
    blob_store: S3Store,
    index_engine: TantivyIndex,
}
```

#### 5. flowbuilder-ai (AI 增强模块)

**职责**: 智能化功能和 AI 集成

-   智能工作流推荐
-   自动优化建议
-   异常检测和预警
-   AI 模型集成

```rust
// flowbuilder-ai AI增强接口
pub trait AIEngine {
    fn recommend_optimization(&self, workflow: &WorkflowDef) -> Vec<OptimizationSuggestion>;
    fn detect_anomaly(&self, metrics: &ExecutionMetrics) -> Option<Anomaly>;
    fn integrate_model(&self, model: &AIModel) -> Result<ModelHandle>;
}

pub struct IntelligentAssistant {
    optimizer: MLOptimizer,
    detector: AnomalyDetector,
    recommender: WorkflowRecommender,
    model_runner: ModelExecutor,
}
```

#### 6. flowbuilder-distributed (分布式扩展)

**职责**: 分布式执行和集群管理

-   集群节点管理
-   分布式任务调度
-   数据分片和复制
-   一致性保证

```rust
// flowbuilder-distributed 分布式接口
pub trait DistributedEngine {
    fn join_cluster(&self, config: &ClusterConfig) -> Result<NodeId>;
    fn distribute_task(&self, task: Task) -> Result<DistributionPlan>;
    fn ensure_consistency(&self) -> Result<ConsistencyState>;
}

pub struct ClusterManager {
    node_registry: NodeRegistry,
    task_distributor: TaskDistributor,
    consensus_engine: RaftConsensus,
    replication_manager: DataReplicator,
}
```

### 模块间通信架构

#### 1. 接口标准化

```rust
// 统一的消息传递接口
pub trait MessageBus {
    fn publish<T: Message>(&self, topic: &str, msg: T) -> Result<()>;
    fn subscribe<T: Message>(&self, topic: &str) -> Result<Receiver<T>>;
}

// 标准化的API网关
pub trait ApiGateway {
    fn route_request(&self, req: Request) -> Result<Response>;
    fn authenticate(&self, token: &str) -> Result<UserContext>;
    fn authorize(&self, user: &UserContext, resource: &str) -> Result<bool>;
}
```

#### 2. 事件驱动架构

```rust
// 事件总线设计
pub struct EventBus {
    channels: HashMap<EventType, Sender<Event>>,
    subscribers: HashMap<EventType, Vec<Subscriber>>,
}

pub enum FlowEvent {
    WorkflowCreated(WorkflowId),
    ExecutionStarted(FlowId),
    TaskCompleted(TaskId),
    ExecutionFailed(FlowId, Error),
    ResourceAllocated(ResourceId),
}
```

#### 3. gRPC 服务网格

```protobuf
// 服务间通信协议
service FlowBuilderCore {
    rpc CompileWorkflow(WorkflowDefinition) returns (ExecutionPlan);
    rpc ValidateWorkflow(WorkflowDefinition) returns (ValidationResult);
}

service FlowRunner {
    rpc ExecuteFlow(ExecutionPlan) returns (stream ExecutionEvent);
    rpc GetFlowStatus(FlowId) returns (FlowStatus);
}

service FlowStorage {
    rpc StoreWorkflow(WorkflowData) returns (StorageResult);
    rpc QueryExecutions(QueryRequest) returns (ExecutionHistory);
}
```

### 模块部署策略

#### 1. 容器化部署

```yaml
# docker-compose.yml
version: "3.8"
services:
    flowbuilder-core:
        image: flowbuilder/core:latest
        ports: ["8080:8080"]
        environment:
            - RUST_LOG=info

    flowrunner:
        image: flowbuilder/runner:latest
        ports: ["8081:8081"]
        deploy:
            replicas: 3

    flowui:
        image: flowbuilder/ui:latest
        ports: ["3000:3000"]
        depends_on: [flowbuilder-core]

    flowbuilder-storage:
        image: flowbuilder/storage:latest
        ports: ["8082:8082"]
        volumes: ["./data:/data"]
```

#### 2. Kubernetes 编排

```yaml
# kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
    name: flowbuilder-suite
spec:
    replicas: 3
    selector:
        matchLabels:
            app: flowbuilder
    template:
        spec:
            containers:
                - name: core
                  image: flowbuilder/core:v1.0
                  resources:
                      requests: { memory: "512Mi", cpu: "500m" }
                      limits: { memory: "1Gi", cpu: "1000m" }
                - name: runner
                  image: flowbuilder/runner:v1.0
                  resources:
                      requests: { memory: "1Gi", cpu: "1000m" }
                      limits: { memory: "2Gi", cpu: "2000m" }
```

### 开发和维护策略

#### 1. 独立版本管理

```toml
# 各模块独立的语义化版本
[workspace.dependencies]
flowbuilder-core = "1.0.0"
flowrunner = "1.1.0"
flowui = "0.9.0"
flowbuilder-storage = "1.0.1"
flowbuilder-ai = "0.8.0"
flowbuilder-distributed = "0.5.0"
```

#### 2. 渐进式集成测试

```rust
// 集成测试框架
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_workflow_execution() {
        let core = FlowBuilderCore::new().await;
        let runner = FlowRunner::connect(&core).await;
        let storage = FlowStorage::new().await;

        // 端到端测试
        let workflow = create_test_workflow();
        let plan = core.compile_workflow(&workflow).await?;
        let result = runner.execute_plan(plan).await?;
        storage.persist_result(&result).await?;

        assert!(result.is_success());
    }
}
```

#### 3. 模块边界清晰化

```rust
// 清晰的模块边界和接口
pub mod flowbuilder_core {
    pub use crate::engine::FlowEngine;
    pub use crate::compiler::WorkflowCompiler;
    // 只暴露必要的公共接口
}

pub mod flowrunner {
    pub use crate::runtime::RuntimeEngine;
    pub use crate::scheduler::TaskScheduler;
    // 运行时专有接口
}
```

### 分布式预留接口设计

### 共识协调层接口 (为分布式演进预留)

#### 1. 共识机制接口

```rust
// 共识层核心接口 - 第一阶段为空实现
pub trait ConsensusEngine: Send + Sync {
    // 提案新的工作流状态变更
    async fn propose_state_change(&self, change: StateChange) -> Result<ProposalId>;

    // 对提案进行投票
    async fn vote_proposal(&self, proposal_id: ProposalId, vote: Vote) -> Result<()>;

    // 提交已达成共识的状态变更
    async fn commit_change(&self, proposal_id: ProposalId) -> Result<()>;

    // 查询提案状态
    async fn get_proposal_status(&self, proposal_id: ProposalId) -> Result<ProposalStatus>;
}

// 第一阶段的空实现 (NoOp Consensus)
pub struct NoOpConsensus;

impl ConsensusEngine for NoOpConsensus {
    async fn propose_state_change(&self, change: StateChange) -> Result<ProposalId> {
        // 单机模式直接返回成功
        Ok(ProposalId::immediate())
    }

    async fn vote_proposal(&self, _proposal_id: ProposalId, _vote: Vote) -> Result<()> {
        // 空实现，直接通过
        Ok(())
    }

    async fn commit_change(&self, _proposal_id: ProposalId) -> Result<()> {
        // 直接提交，无需共识
        Ok(())
    }

    async fn get_proposal_status(&self, _proposal_id: ProposalId) -> Result<ProposalStatus> {
        Ok(ProposalStatus::Committed)
    }
}

// 状态变更类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChange {
    WorkflowCreated { workflow_id: WorkflowId, definition: WorkflowDef },
    TaskStatusUpdate { task_id: TaskId, status: TaskStatus },
    ResourceAllocation { resource_id: ResourceId, allocation: ResourceAllocation },
    NodeJoined { node_id: NodeId, node_info: NodeInfo },
    NodeLeft { node_id: NodeId },
}

#[derive(Debug, Clone)]
pub enum Vote {
    Approve,
    Reject(String), // 包含拒绝原因
}

#[derive(Debug, Clone)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Committed,
    Timeout,
}
```

#### 2. 节点管理接口

```rust
// 节点发现和管理接口
pub trait NodeManager: Send + Sync {
    // 注册当前节点
    async fn register_node(&self, node_info: NodeInfo) -> Result<NodeId>;

    // 发现其他节点
    async fn discover_nodes(&self) -> Result<Vec<NodeInfo>>;

    // 监控节点健康状态
    async fn monitor_node_health(&self, node_id: NodeId) -> Result<NodeHealth>;

    // 节点离线处理
    async fn handle_node_offline(&self, node_id: NodeId) -> Result<()>;

    // 获取集群拓扑
    async fn get_cluster_topology(&self) -> Result<ClusterTopology>;
}

// 第一阶段的单机实现
pub struct SingleNodeManager {
    local_node: NodeInfo,
}

impl NodeManager for SingleNodeManager {
    async fn register_node(&self, node_info: NodeInfo) -> Result<NodeId> {
        // 单机模式只有一个节点
        Ok(self.local_node.id)
    }

    async fn discover_nodes(&self) -> Result<Vec<NodeInfo>> {
        // 只返回本地节点
        Ok(vec![self.local_node.clone()])
    }

    async fn monitor_node_health(&self, _node_id: NodeId) -> Result<NodeHealth> {
        Ok(NodeHealth::Healthy)
    }

    async fn handle_node_offline(&self, _node_id: NodeId) -> Result<()> {
        // 单机模式无需处理
        Ok(())
    }

    async fn get_cluster_topology(&self) -> Result<ClusterTopology> {
        Ok(ClusterTopology::single_node(self.local_node.clone()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub address: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub resources: ResourceCapacity,
    pub region: Option<String>,
    pub zone: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NodeHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Offline,
}

#[derive(Debug, Clone)]
pub struct ClusterTopology {
    pub nodes: Vec<NodeInfo>,
    pub leader: Option<NodeId>,
    pub regions: HashMap<String, Vec<NodeId>>,
}
```

#### 3. 分布式状态同步接口

```rust
// 状态同步接口
pub trait StateSynchronizer: Send + Sync {
    // 同步工作流状态
    async fn sync_workflow_state(&self, workflow_id: WorkflowId) -> Result<WorkflowState>;

    // 同步任务状态
    async fn sync_task_state(&self, task_id: TaskId) -> Result<TaskState>;

    // 广播状态变更
    async fn broadcast_state_change(&self, change: StateChange) -> Result<()>;

    // 订阅状态变更
    async fn subscribe_state_changes(&self) -> Result<Receiver<StateChange>>;

    // 检查状态一致性
    async fn check_consistency(&self) -> Result<ConsistencyReport>;
}

// 第一阶段的本地实现
pub struct LocalStateSynchronizer {
    state_store: Arc<dyn StateStore>,
    event_bus: Arc<dyn EventBus>,
}

impl StateSynchronizer for LocalStateSynchronizer {
    async fn sync_workflow_state(&self, workflow_id: WorkflowId) -> Result<WorkflowState> {
        // 本地直接从存储读取
        self.state_store.get_workflow_state(workflow_id).await
    }

    async fn sync_task_state(&self, task_id: TaskId) -> Result<TaskState> {
        self.state_store.get_task_state(task_id).await
    }

    async fn broadcast_state_change(&self, change: StateChange) -> Result<()> {
        // 本地事件总线广播
        self.event_bus.publish("state_change", change).await
    }

    async fn subscribe_state_changes(&self) -> Result<Receiver<StateChange>> {
        self.event_bus.subscribe("state_change").await
    }

    async fn check_consistency(&self) -> Result<ConsistencyReport> {
        // 单机模式始终一致
        Ok(ConsistencyReport::consistent())
    }
}
```

#### 4. 分布式锁接口

```rust
// 分布式锁接口
pub trait DistributedLock: Send + Sync {
    // 获取锁
    async fn acquire_lock(&self, resource: &str, ttl: Duration) -> Result<LockHandle>;

    // 释放锁
    async fn release_lock(&self, handle: LockHandle) -> Result<()>;

    // 续期锁
    async fn renew_lock(&self, handle: &LockHandle, ttl: Duration) -> Result<()>;

    // 检查锁状态
    async fn check_lock(&self, resource: &str) -> Result<Option<LockInfo>>;
}

// 第一阶段的本地锁实现
pub struct LocalLock {
    locks: Arc<RwLock<HashMap<String, LockInfo>>>,
}

impl DistributedLock for LocalLock {
    async fn acquire_lock(&self, resource: &str, ttl: Duration) -> Result<LockHandle> {
        let mut locks = self.locks.write().await;

        // 检查是否已被锁定
        if let Some(existing) = locks.get(resource) {
            if !existing.is_expired() {
                return Err(anyhow!("Resource already locked"));
            }
        }

        let handle = LockHandle::new();
        let lock_info = LockInfo {
            handle: handle.clone(),
            owner: "local".to_string(),
            acquired_at: Instant::now(),
            ttl,
        };

        locks.insert(resource.to_string(), lock_info);
        Ok(handle)
    }

    async fn release_lock(&self, handle: LockHandle) -> Result<()> {
        let mut locks = self.locks.write().await;
        locks.retain(|_, lock_info| lock_info.handle != handle);
        Ok(())
    }

    async fn renew_lock(&self, handle: &LockHandle, ttl: Duration) -> Result<()> {
        let mut locks = self.locks.write().await;
        for lock_info in locks.values_mut() {
            if lock_info.handle == *handle {
                lock_info.acquired_at = Instant::now();
                lock_info.ttl = ttl;
                return Ok(());
            }
        }
        Err(anyhow!("Lock not found"))
    }

    async fn check_lock(&self, resource: &str) -> Result<Option<LockInfo>> {
        let locks = self.locks.read().await;
        Ok(locks.get(resource).cloned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LockHandle {
    id: Uuid,
}

impl LockHandle {
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub handle: LockHandle,
    pub owner: String,
    pub acquired_at: Instant,
    pub ttl: Duration,
}

impl LockInfo {
    fn is_expired(&self) -> bool {
        self.acquired_at.elapsed() > self.ttl
    }
}
```

#### 5. 分布式事件总线接口

```rust
// 分布式事件总线接口
pub trait DistributedEventBus: Send + Sync {
    // 发布事件到集群
    async fn publish_cluster(&self, topic: &str, event: Event) -> Result<()>;

    // 订阅集群事件
    async fn subscribe_cluster(&self, topic: &str) -> Result<Receiver<Event>>;

    // 发布本地事件
    async fn publish_local(&self, topic: &str, event: Event) -> Result<()>;

    // 订阅本地事件
    async fn subscribe_local(&self, topic: &str) -> Result<Receiver<Event>>;

    // 获取事件统计
    async fn get_event_stats(&self) -> Result<EventStats>;
}

// 第一阶段的本地事件总线
pub struct LocalEventBus {
    channels: Arc<RwLock<HashMap<String, Vec<Sender<Event>>>>>,
    stats: Arc<AtomicU64>,
}

impl DistributedEventBus for LocalEventBus {
    async fn publish_cluster(&self, topic: &str, event: Event) -> Result<()> {
        // 第一阶段等同于本地发布
        self.publish_local(topic, event).await
    }

    async fn subscribe_cluster(&self, topic: &str) -> Result<Receiver<Event>> {
        // 第一阶段等同于本地订阅
        self.subscribe_local(topic).await
    }

    async fn publish_local(&self, topic: &str, event: Event) -> Result<()> {
        let channels = self.channels.read().await;
        if let Some(senders) = channels.get(topic) {
            for sender in senders {
                let _ = sender.send(event.clone()).await;
            }
        }
        self.stats.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn subscribe_local(&self, topic: &str) -> Result<Receiver<Event>> {
        let (sender, receiver) = mpsc::channel(1000);
        let mut channels = self.channels.write().await;
        channels.entry(topic.to_string()).or_default().push(sender);
        Ok(receiver)
    }

    async fn get_event_stats(&self) -> Result<EventStats> {
        Ok(EventStats {
            total_events: self.stats.load(Ordering::Relaxed),
            local_events: self.stats.load(Ordering::Relaxed),
            cluster_events: 0, // 第一阶段为0
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EventStats {
    pub total_events: u64,
    pub local_events: u64,
    pub cluster_events: u64,
}
```

### 接口集成策略

#### 1. 依赖注入设计

```rust
// 统一的服务容器
pub struct ServiceContainer {
    // 核心服务
    pub consensus: Arc<dyn ConsensusEngine>,
    pub node_manager: Arc<dyn NodeManager>,
    pub state_sync: Arc<dyn StateSynchronizer>,
    pub distributed_lock: Arc<dyn DistributedLock>,
    pub event_bus: Arc<dyn DistributedEventBus>,

    // 配置信息
    pub config: Arc<SystemConfig>,
}

impl ServiceContainer {
    // 第一阶段：创建单机版本
    pub fn new_single_node(config: SystemConfig) -> Self {
        let node_info = NodeInfo::local();

        Self {
            consensus: Arc::new(NoOpConsensus),
            node_manager: Arc::new(SingleNodeManager::new(node_info)),
            state_sync: Arc::new(LocalStateSynchronizer::new()),
            distributed_lock: Arc::new(LocalLock::new()),
            event_bus: Arc::new(LocalEventBus::new()),
            config: Arc::new(config),
        }
    }

    // 后续阶段：创建分布式版本
    pub async fn new_distributed(config: SystemConfig) -> Result<Self> {
        // 在分布式阶段替换为真实实现
        todo!("Implement in distributed phase")
    }
}
```

#### 2. 配置驱动切换

```rust
// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub mode: DeploymentMode,
    pub consensus: ConsensusConfig,
    pub networking: NetworkConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    SingleNode,
    Cluster { nodes: Vec<String> },
    P2P { bootstrap_nodes: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: ConsensusAlgorithm,
    pub timeout: Duration,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    NoOp,           // 第一阶段
    Raft,           // 分布式阶段
    Byzantine,      // 去中心化阶段
}
```

这种设计确保了第一阶段可以专注于单机性能优化，同时为后续的分布式扩展预留了完整的接口，实现平滑演进。

---
