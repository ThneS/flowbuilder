# FlowBuilder API 参考

## 核心 API

### DynamicFlowExecutor

统一的流程执行器，实现新的分层架构。

```rust
pub struct DynamicFlowExecutor {
    config: WorkflowConfig,
    parser: YamlConfigParser,
    orchestrator: EnhancedFlowOrchestrator,
    executor: EnhancedTaskExecutor,
    evaluator: ExpressionEvaluator,
}
```

#### 方法

##### new

```rust
pub fn new(config: WorkflowConfig) -> Result<Self>
```

创建新的动态流程执行器。

##### from_yaml

```rust
pub fn from_yaml(yaml_content: &str) -> Result<Self>
```

从 YAML 字符串创建执行器。

##### execute

```rust
pub async fn execute(&mut self, context: SharedContext) -> Result<ExecutionResult>
```

执行工作流。

##### get_execution_plan_preview

```rust
pub fn get_execution_plan_preview(&self) -> Result<ExecutionPlan>
```

获取执行计划预览（不执行）。

##### analyze_workflow_complexity

```rust
pub fn analyze_workflow_complexity(&self) -> Result<ExecutionComplexity>
```

分析工作流复杂度。

### YamlConfigParser

YAML 配置解析器。

```rust
pub struct YamlConfigParser {
    config: WorkflowConfig,
}
```

#### 方法

##### new

```rust
pub fn new(config: WorkflowConfig) -> Self
```

创建新的配置解析器。

##### parse

```rust
pub fn parse(&self) -> Result<Vec<ExecutionNode>>
```

解析配置，生成执行节点列表。

##### parse_full

```rust
pub fn parse_full(&self) -> Result<ParseResult>
```

解析配置并返回完整结果。

### EnhancedFlowOrchestrator

增强的流程编排器。

```rust
pub struct EnhancedFlowOrchestrator {
    config: OrchestratorConfig,
}
```

#### 方法

##### new

```rust
pub fn new() -> Self
```

创建新的编排器。

##### create_execution_plan

```rust
pub fn create_execution_plan(
    &self,
    nodes: Vec<ExecutionNode>,
    env_vars: HashMap<String, serde_yaml::Value>,
    flow_vars: HashMap<String, serde_yaml::Value>,
    workflow_name: String,
    workflow_version: String,
) -> Result<ExecutionPlan>
```

从节点列表创建执行计划。

##### analyze_complexity

```rust
pub fn analyze_complexity(&self, plan: &ExecutionPlan) -> ExecutionComplexity
```

分析执行计划的复杂度。

### EnhancedTaskExecutor

增强的任务执行器。

```rust
pub struct EnhancedTaskExecutor {
    config: ExecutorConfig,
}
```

#### 方法

##### new

```rust
pub fn new() -> Self
```

创建新的任务执行器。

##### execute_plan

```rust
pub async fn execute_plan(
    &mut self,
    plan: ExecutionPlan,
    context: SharedContext,
) -> Result<ExecutionResult>
```

执行执行计划。

## 数据结构

### ExecutionPlan

执行计划，包含执行阶段和元数据。

```rust
pub struct ExecutionPlan {
    pub phases: Vec<ExecutionPhase>,
    pub metadata: PlanMetadata,
    pub env_vars: HashMap<String, serde_yaml::Value>,
    pub flow_vars: HashMap<String, serde_yaml::Value>,
}
```

### ExecutionNode

执行节点，表示单个任务或动作。

```rust
pub struct ExecutionNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_type: NodeType,
    pub action_spec: ActionSpec,
    pub dependencies: Vec<String>,
    pub retry_config: Option<RetryConfig>,
    pub timeout_config: Option<TimeoutConfig>,
}
```

### WorkflowConfig

工作流配置，从 YAML 解析得到。

```rust
pub struct WorkflowConfig {
    pub workflow: WorkflowInfo,
}

pub struct WorkflowInfo {
    pub version: String,
    pub env: HashMap<String, String>,
    pub vars: HashMap<String, serde_yaml::Value>,
    pub tasks: Vec<TaskWrapper>,
}
```

### ExecutionResult

执行结果，包含成功状态和详细信息。

```rust
pub struct ExecutionResult {
    pub success: bool,
    pub phase_results: Vec<PhaseResult>,
    pub total_duration: std::time::Duration,
    pub nodes_executed: usize,
    pub errors: Vec<String>,
    pub stats: ExecutionStats,
}
```

## 错误处理

所有 API 方法都返回 `Result<T, anyhow::Error>`，确保错误信息的完整性和可追溯性。

### 常见错误类型

-   **配置解析错误**: YAML 格式错误或缺少必要字段
-   **执行计划创建错误**: 节点依赖循环或无效配置
-   **任务执行错误**: 任务执行失败或超时
-   **上下文错误**: 上下文访问失败或状态不一致

## 特性 (Features)

### 默认特性

-   `core`: 核心流程构建功能
-   `yaml`: YAML 配置支持
-   `runtime`: 运行时增强功能

### 可选特性

所有功能默认启用，项目专注于核心工作流执行功能。

## 示例

查看 `examples/new_architecture_demo.rs` 获取完整的 API 使用示例。
