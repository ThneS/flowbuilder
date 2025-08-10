# FlowBuilder 快速开始指南

## 🚀 立即开始：单人+AI 开发模式

### 预备条件

```bash
# 开发环境准备
rustup update stable
cargo install cargo-watch just
npm install -g @tauri-apps/cli

# AI工具安装
# 1. 安装Cursor IDE: https://cursor.sh
# 2. 配置GitHub Copilot
# 3. 注册Claude/ChatGPT账号
```

## 第一天：项目初始化

### 1. 创建项目结构

```bash
# AI提示词：请帮我创建一个Rust工作流引擎的项目结构
cargo new --workspace flowbuilder
cd flowbuilder

# 添加子模块
cargo new --lib flowbuilder-core
cargo new --lib flowbuilder-runtime
cargo new --bin flowbuilder-cli
cargo new --lib flowbuilder-web
```

### 2. 配置 Cargo.toml

```toml
# 使用AI生成完整的workspace配置
[workspace]
members = [
    "flowbuilder-core",
    "flowbuilder-runtime",
    "flowbuilder-cli",
    "flowbuilder-web",
]

[workspace.dependencies]
# AI建议的依赖包
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
axum = "0.7"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 3. 核心数据结构设计

**提示 AI 生成核心结构**:

```
请设计一个Rust工作流引擎的核心数据结构，要求：
1. 支持YAML配置解析
2. 任务依赖图表示
3. 异步执行接口
4. 错误处理和重试
5. 进度回调机制

请提供完整的Rust代码。
```

**预期输出** (flowbuilder-core/src/lib.rs):

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub tasks: Vec<Task>,
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub task_type: TaskType,
    pub depends_on: Vec<String>,
    pub config: TaskConfig,
    pub retry: Option<RetryConfig>,
    pub timeout: Option<u64>,
}

// ... AI生成更多结构
```

## 第二天：工作流解析器

### 1. YAML 解析实现

**AI 提示**:

```
实现一个Rust YAML工作流解析器，要求：
- 解析YAML文件为Workflow结构
- 验证工作流定义的正确性
- 检查任务依赖的循环引用
- 提供详细的错误信息

请提供完整实现和测试。
```

### 2. 依赖图构建

```rust
// AI生成的依赖图算法
pub struct DependencyGraph {
    nodes: HashMap<String, TaskNode>,
    edges: Vec<(String, String)>,
}

impl DependencyGraph {
    pub fn from_workflow(workflow: &Workflow) -> Result<Self> {
        // AI实现拓扑排序和循环检测
    }

    pub fn execution_order(&self) -> Result<Vec<Vec<String>>> {
        // AI实现并行执行计划生成
    }
}
```

## 第三天：任务执行器

### 1. 执行器接口设计

```rust
// AI生成的执行器trait
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: &Task, context: &ExecutionContext) -> Result<TaskOutput>;
    fn supports_type(&self) -> TaskType;
}

// 具体执行器实现
pub struct ScriptExecutor;
pub struct HttpExecutor;
pub struct ShellExecutor;
```

### 2. 基础执行器实现

**AI 提示**:

```
实现Rust异步任务执行器，支持以下类型：
1. ScriptExecutor: 执行Python/JavaScript脚本
2. HttpExecutor: 发送HTTP请求
3. ShellExecutor: 执行shell命令

每个执行器要求：
- 异步执行
- 超时控制
- 错误处理
- 进度回调
- 资源清理

请提供完整实现。
```

## 第四天：CLI 工具

### 1. 命令行界面

```rust
// flowbuilder-cli/src/main.rs
// AI生成的CLI应用
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flowbuilder")]
#[command(about = "高性能工作流引擎")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { file: String },
    Validate { file: String },
    List,
    Status { id: String },
}
```

### 2. 基础命令实现

**AI 提示**:

```
实现FlowBuilder CLI工具，支持命令：
- run <file>: 执行工作流文件
- validate <file>: 验证工作流语法
- list: 列出所有工作流
- status <id>: 查询执行状态

使用clap库，提供良好的用户体验。
```

## 第五天：Web API

### 1. Axum 服务器

```rust
// flowbuilder-web/src/main.rs
// AI生成的Web服务
use axum::{
    routing::{get, post},
    Router, Json, extract::Path,
};

async fn create_workflow(Json(workflow): Json<Workflow>) -> Result<Json<WorkflowId>, AppError> {
    // AI实现
}

async fn execute_workflow(Path(id): Path<String>) -> Result<Json<ExecutionResult>, AppError> {
    // AI实现
}

fn app() -> Router {
    Router::new()
        .route("/workflows", post(create_workflow))
        .route("/workflows/:id/execute", post(execute_workflow))
        .route("/workflows/:id/status", get(get_workflow_status))
}
```

## 第六天：前端界面

### 1. React 组件生成

**v0.dev 提示**:

```
创建一个现代化的工作流管理界面，包含：

1. 顶部导航栏
   - Logo和标题
   - 用户菜单
   - 主题切换

2. 侧边栏
   - 工作流列表
   - 执行历史
   - 系统设置

3. 主内容区
   - 工作流卡片网格
   - 每个卡片显示：名称、状态、最后执行时间、执行按钮
   - 支持搜索和过滤

4. 详情页面
   - 工作流DAG图（使用ReactFlow）
   - 任务详情面板
   - 执行日志

使用Tailwind CSS，深色主题，现代设计风格。
```

### 2. API 客户端

```typescript
// AI生成的TypeScript客户端
export class FlowBuilderAPI {
    constructor(private baseURL: string) {}

    async createWorkflow(workflow: Workflow): Promise<WorkflowId> {
        // AI实现
    }

    async executeWorkflow(id: string): Promise<ExecutionResult> {
        // AI实现
    }

    async getWorkflowStatus(id: string): Promise<WorkflowStatus> {
        // AI实现
    }
}
```

## 第七天：桌面应用

### 1. Tauri 集成

```bash
# 创建Tauri应用
cd flowbuilder-web
npm install
npm install -D @tauri-apps/cli
npx tauri init

# AI帮助配置tauri.conf.json
```

### 2. 原生功能集成

```rust
// src-tauri/src/main.rs
// AI生成的Tauri命令
#[tauri::command]
async fn load_workflow_file(path: String) -> Result<Workflow, String> {
    // AI实现文件读取和解析
}

#[tauri::command]
async fn execute_workflow_local(workflow: Workflow) -> Result<ExecutionResult, String> {
    // AI实现本地执行
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_workflow_file,
            execute_workflow_local
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 一周成果检查

### 功能验证清单

-   [ ] **工作流解析**: 正确解析 YAML 文件
-   [ ] **依赖检查**: 检测循环依赖和无效引用
-   [ ] **任务执行**: 支持 script、http、shell 任务
-   [ ] **CLI 工具**: 基础命令行操作
-   [ ] **Web API**: REST 接口完整
-   [ ] **前端界面**: 可视化操作界面
-   [ ] **桌面应用**: 跨平台桌面版本

### 性能测试

```yaml
# 测试工作流 (test-workflow.yaml)
name: "performance-test"
version: "1.0"
tasks:
    - name: "parallel-1"
      type: "script"
      config:
          language: "python"
          code: "import time; time.sleep(1); print('Task 1 done')"

    - name: "parallel-2"
      type: "script"
      config:
          language: "python"
          code: "import time; time.sleep(1); print('Task 2 done')"

    - name: "sequential"
      type: "shell"
      depends_on: ["parallel-1", "parallel-2"]
      config:
          command: "echo 'All parallel tasks completed'"
```

```bash
# 性能验证命令
cargo run --bin flowbuilder-cli -- run test-workflow.yaml
# 预期: 并行任务1秒完成，而非2秒
```

## 快速部署

### Docker 构建

```dockerfile
# AI生成优化的Dockerfile
FROM rust:1.75-alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/flowbuilder-cli /usr/local/bin/
COPY --from=builder /app/target/release/flowbuilder-web /usr/local/bin/
EXPOSE 3000
CMD ["flowbuilder-web"]
```

### 云平台部署

```bash
# Railway部署
railway login
railway init
railway up

# 或 Fly.io部署
fly auth login
fly launch
fly deploy
```

## 后续迭代方向

### 第二周目标

-   [ ] 数据持久化 (SQLite)
-   [ ] 实时状态更新 (WebSocket)
-   [ ] 错误重试机制
-   [ ] 简单的调度功能
-   [ ] 基础监控面板

### 第三周目标

-   [ ] 插件系统框架
-   [ ] 更多任务类型
-   [ ] 性能优化
-   [ ] 压力测试
-   [ ] 用户文档

### 第四周目标

-   [ ] 生产环境部署
-   [ ] 安全性加固
-   [ ] 备份恢复
-   [ ] 演示视频
-   [ ] 开源发布

通过这个指南，您可以在一周内用 AI 协作完成一个可用的工作流引擎原型！

## 第二天补充：分布式接口预留

### 共识层接口设计

**AI 提示**:

```
设计FlowBuilder的分布式共识接口，要求：
1. 第一阶段使用NoOp实现（单机模式）
2. 预留Raft/Byzantine共识算法接口
3. 支持状态变更提案和投票
4. 配置驱动的部署模式切换

请生成完整的Rust trait定义和NoOp实现。
```

**预期输出**:

```rust
// flowbuilder-core/src/consensus.rs
#[async_trait]
pub trait ConsensusEngine: Send + Sync {
    async fn propose(&self, change: StateChange) -> Result<ProposalId>;
    async fn vote(&self, proposal_id: ProposalId, vote: Vote) -> Result<()>;
    async fn commit(&self, proposal_id: ProposalId) -> Result<()>;
    async fn get_leader(&self) -> Result<Option<NodeId>>;
}

// 第一阶段NoOp实现
pub struct NoOpConsensus;

impl ConsensusEngine for NoOpConsensus {
    async fn propose(&self, _change: StateChange) -> Result<ProposalId> {
        Ok(ProposalId::immediate()) // 直接通过
    }

    async fn vote(&self, _proposal_id: ProposalId, _vote: Vote) -> Result<()> {
        Ok(()) // 无需投票
    }

    async fn commit(&self, _proposal_id: ProposalId) -> Result<()> {
        Ok(()) // 立即提交
    }

    async fn get_leader(&self) -> Result<Option<NodeId>> {
        Ok(Some(NodeId::local())) // 自己是leader
    }
}

// 状态变更定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChange {
    WorkflowCreated { workflow_id: WorkflowId, definition: WorkflowDef },
    TaskStarted { task_id: TaskId, node_id: NodeId },
    TaskCompleted { task_id: TaskId, result: TaskResult },
    // ... 更多状态变更类型
}
```

### 节点管理接口

```rust
// 节点管理接口
#[async_trait]
pub trait NodeManager: Send + Sync {
    async fn register(&self, node_info: NodeInfo) -> Result<NodeId>;
    async fn discover(&self) -> Result<Vec<NodeInfo>>;
    async fn health_check(&self, node_id: NodeId) -> Result<NodeHealth>;
    async fn topology(&self) -> Result<ClusterTopology>;
}

// 第一阶段单机实现
pub struct LocalNodeManager {
    local_node: NodeInfo,
}

impl NodeManager for LocalNodeManager {
    async fn register(&self, _node_info: NodeInfo) -> Result<NodeId> {
        Ok(self.local_node.id) // 总是返回本地节点
    }

    async fn discover(&self) -> Result<Vec<NodeInfo>> {
        Ok(vec![self.local_node.clone()]) // 只有本地节点
    }

    async fn health_check(&self, _node_id: NodeId) -> Result<NodeHealth> {
        Ok(NodeHealth::Healthy) // 本地节点总是健康
    }

    async fn topology(&self) -> Result<ClusterTopology> {
        Ok(ClusterTopology::single_node(self.local_node.clone()))
    }
}
```

### 服务容器集成

```rust
// 统一的服务容器
pub struct ServiceContainer {
    pub consensus: Arc<dyn ConsensusEngine>,
    pub node_manager: Arc<dyn NodeManager>,
    pub workflow_engine: Arc<WorkflowEngine>,
    pub config: Arc<SystemConfig>,
}

impl ServiceContainer {
    // 第一阶段：单机模式
    pub fn new_single_node() -> Self {
        Self {
            consensus: Arc::new(NoOpConsensus),
            node_manager: Arc::new(LocalNodeManager::new()),
            workflow_engine: Arc::new(WorkflowEngine::new()),
            config: Arc::new(SystemConfig::single_node()),
        }
    }

    // 第二阶段：集群模式（预留）
    pub async fn new_cluster(config: SystemConfig) -> Result<Self> {
        todo!("Implement in distributed phase")
    }
}
```

这样设计确保了第一阶段专注单机性能，同时为分布式扩展预留完整接口。

---

## 附录：Feature 组合速查

| 场景            | Cargo features                                                 | import 示例                             |
| --------------- | -------------------------------------------------------------- | --------------------------------------- |
| 最小核心        | core                                                           | `use flowbuilder::prelude::*;`          |
| 并行执行        | core, runtime, parallel                                        | `use flowbuilder::runtime::prelude::*;` |
| 重试 + 并行     | core, runtime, parallel, retry                                 | 同上                                    |
| YAML 动态加载   | yaml                                                           | `use flowbuilder::yaml::prelude::*;`    |
| YAML + 高级执行 | yaml, runtime                                                  | 两个 prelude 分别导入                   |
| 全量调试        | yaml, runtime, parallel, retry, perf-metrics, detailed-logging | 同上                                    |

最小体积建议：不需要并行/重试/指标时仅保留 `core` 或 `core + runtime`。

```toml
[dependencies]
flowbuilder = { version = "0.1.0", default-features = false, features = ["core", "runtime"] }
```

> 注意：`yaml` 不再自动 re-export `runtime`，需要同时使用请显式启用并分别导入。
