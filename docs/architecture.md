# FlowBuilder 架构设计

## 总体架构

FlowBuilder 采用分层架构设计，将工作流执行分为三个清晰的层次：

```
┌─────────────────────────────────────────────────────────────┐
│                    YAML 配置文件                            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ workflow:                                           │    │
│  │   version: "1.0"                                    │    │
│  │   tasks:                                            │    │
│  │     - task: ...                                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                 配置解析器层                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │           YamlConfigParser                          │    │
│  │  • 解析 YAML 配置                                   │    │
│  │  • 生成执行节点                                     │    │
│  │  • 验证配置完整性                                   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                 流程编排器层                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │        EnhancedFlowOrchestrator                     │    │
│  │  • 创建执行计划                                     │    │
│  │  • 优化执行顺序                                     │    │
│  │  • 分析工作流复杂度                                 │    │
│  │  • 处理依赖关系                                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                 任务执行器层                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         EnhancedTaskExecutor                        │    │
│  │  • 执行具体任务                                     │    │
│  │  • 并行执行控制                                     │    │
│  │  • 重试和超时处理                                   │    │
│  │  • 错误恢复机制                                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. 配置解析器 (YamlConfigParser)

**职责**：

-   解析 YAML 配置文件
-   验证配置格式和完整性
-   生成结构化的执行节点
-   提取环境变量和流程变量

**核心接口**：

```rust
impl ConfigParser for YamlConfigParser {
    fn parse(&self) -> Result<Vec<ExecutionNode>>;
    fn validate(&self) -> Result<()>;
    fn get_env_vars(&self) -> HashMap<String, String>;
    fn get_flow_vars(&self) -> HashMap<String, serde_yaml::Value>;
}
```

### 2. 流程编排器 (EnhancedFlowOrchestrator)

**职责**：

-   根据节点依赖关系创建执行计划
-   优化执行顺序，最大化并行度
-   分析工作流复杂度
-   处理阶段划分和执行模式

**核心接口**：

```rust
impl FlowPlanner for EnhancedFlowOrchestrator {
    fn create_execution_plan(&self, nodes: Vec<ExecutionNode>) -> Result<ExecutionPlan>;
    fn analyze_complexity(&self, plan: &ExecutionPlan) -> ExecutionComplexity;
    fn optimize_plan(&self, plan: ExecutionPlan) -> ExecutionPlan;
}
```

### 3. 任务执行器 (EnhancedTaskExecutor)

**职责**：

-   执行具体的任务和动作
-   管理并行执行和资源控制
-   处理重试、超时和错误恢复
-   收集执行指标和状态

**核心接口**：

```rust
impl Executor for EnhancedTaskExecutor {
    type Input = ExecutionPlan;
    type Output = ExecutionResult;
    type Error = anyhow::Error;

    async fn execute(&mut self, plan: ExecutionPlan) -> Result<ExecutionResult>;
    async fn stop(&mut self) -> Result<()>;
    fn status(&self) -> ExecutorStatus;
}
```

## 数据流

### 1. 配置解析阶段

```
YAML 配置 → YamlConfigParser → ExecutionNode[]
                                      ↓
                              环境变量 + 流程变量
```

### 2. 执行计划生成阶段

```
ExecutionNode[] → EnhancedFlowOrchestrator → ExecutionPlan
                                                  ↓
                                         优化的执行阶段
```

### 3. 任务执行阶段

```
ExecutionPlan → EnhancedTaskExecutor → ExecutionResult
                                            ↓
                                      执行统计 + 错误信息
```

## 核心数据结构

### ExecutionPlan

```rust
pub struct ExecutionPlan {
    /// 执行阶段列表，按依赖顺序排列
    pub phases: Vec<ExecutionPhase>,
    /// 计划元数据
    pub metadata: PlanMetadata,
    /// 环境变量
    pub env_vars: HashMap<String, serde_yaml::Value>,
    /// 流程变量
    pub flow_vars: HashMap<String, serde_yaml::Value>,
}
```

### ExecutionPhase

```rust
pub struct ExecutionPhase {
    /// 阶段 ID
    pub id: String,
    /// 阶段中的节点
    pub nodes: Vec<ExecutionNode>,
    /// 执行模式（串行/并行）
    pub execution_mode: PhaseExecutionMode,
    /// 阶段级别配置
    pub config: PhaseConfig,
}
```

### ExecutionNode

```rust
pub struct ExecutionNode {
    /// 节点唯一标识
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点描述
    pub description: Option<String>,
    /// 节点类型
    pub node_type: NodeType,
    /// 动作规范
    pub action_spec: ActionSpec,
    /// 依赖的其他节点
    pub dependencies: Vec<String>,
    /// 重试配置
    pub retry_config: Option<RetryConfig>,
    /// 超时配置
    pub timeout_config: Option<TimeoutConfig>,
}
```

## 设计原则

### 1. 关注点分离

每一层都有明确的职责：

-   **配置解析器**：只负责解析和验证
-   **流程编排器**：只负责计划和优化
-   **任务执行器**：只负责执行和控制

### 2. 接口驱动

通过标准接口定义各层的交互：

-   `ConfigParser` trait
-   `FlowPlanner` trait
-   `Executor` trait

### 3. 可测试性

每一层都可以独立测试：

-   解析器可以独立测试配置解析逻辑
-   编排器可以独立测试计划生成和优化
-   执行器可以独立测试任务执行逻辑

### 4. 可扩展性

通过统一接口支持多种实现：

-   可以有不同的配置格式解析器
-   可以有不同的编排策略
-   可以有不同的执行器实现

## 配置格式

### YAML 配置结构

```yaml
workflow:
    version: "1.0" # 工作流版本
    env: # 环境变量
        ENVIRONMENT: "production"
        LOG_LEVEL: "info"
    vars: # 流程变量
        max_retries: 3
        timeout: 30
    tasks: # 任务列表
        - task:
              id: "task1" # 任务 ID
              name: "任务名称" # 任务名称
              description: "任务描述" # 任务描述
              actions: # 动作列表
                  - action:
                        id: "action1" # 动作 ID
                        name: "动作名称" # 动作名称
                        type: "builtin" # 动作类型
                        flow: # 流程控制
                            retry: # 重试配置
                                max_retries: 2
                                delay: 1000
                            timeout: # 超时配置
                                duration: 5000
                        outputs: # 输出配置
                            key: "value"
                        parameters: # 参数配置
                            param: "value"
```

## 性能特性

### 1. 并行执行

-   自动分析节点依赖关系
-   最大化并行执行机会
-   支持阶段内并行和阶段间串行

### 2. 资源控制

-   可配置的并发限制
-   内存和 CPU 使用优化
-   支持背压和流量控制

### 3. 错误恢复

-   多层次的错误处理
-   可配置的重试策略
-   支持部分失败继续执行

这种分层架构确保了 FlowBuilder 的高性能、可扩展性和易维护性。
