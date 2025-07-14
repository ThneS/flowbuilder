//! # FlowBuilder Core - 执行计划和节点定义
//!
//! 定义流程执行的核心数据结构和接口

use anyhow::Result;
use std::collections::HashMap;

/// 执行计划 - 编排器生成的执行顺序
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 执行阶段列表（按顺序执行）
    pub phases: Vec<ExecutionPhase>,
    /// 全局环境变量
    pub env_vars: HashMap<String, serde_yaml::Value>,
    /// 全局流程变量
    pub flow_vars: HashMap<String, serde_yaml::Value>,
    /// 计划元数据
    pub metadata: PlanMetadata,
}

/// 执行阶段 - 可以串行或并行执行的任务组
#[derive(Debug, Clone)]
pub struct ExecutionPhase {
    /// 阶段ID
    pub id: String,
    /// 阶段名称
    pub name: String,
    /// 执行模式
    pub execution_mode: PhaseExecutionMode,
    /// 该阶段包含的执行节点
    pub nodes: Vec<ExecutionNode>,
    /// 阶段条件
    pub condition: Option<String>,
}

/// 阶段执行模式
#[derive(Debug, Clone)]
pub enum PhaseExecutionMode {
    /// 串行执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 条件执行
    Conditional { condition: String },
}

/// 执行节点 - 最小的执行单元
#[derive(Debug, Clone)]
pub struct ExecutionNode {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 关联的动作定义
    pub action_spec: ActionSpec,
    /// 依赖的节点ID列表
    pub dependencies: Vec<String>,
    /// 节点执行条件
    pub condition: Option<String>,
    /// 节点优先级
    pub priority: u32,
    /// 重试配置
    pub retry_config: Option<RetryConfig>,
    /// 超时配置
    pub timeout_config: Option<TimeoutConfig>,
}

/// 节点类型
#[derive(Debug, Clone)]
pub enum NodeType {
    /// 动作节点
    Action,
    /// 条件节点
    Condition,
    /// 分支节点
    Branch,
    /// 循环节点
    Loop,
    /// 子流程节点
    Subprocess,
}

/// 动作规格
#[derive(Debug, Clone)]
pub struct ActionSpec {
    /// 动作类型
    pub action_type: String,
    /// 动作参数
    pub parameters: HashMap<String, serde_yaml::Value>,
    /// 动作输出
    pub outputs: HashMap<String, serde_yaml::Value>,
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub delay: u64,
    /// 重试策略
    pub strategy: RetryStrategy,
}

/// 重试策略
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// 固定延迟
    Fixed,
    /// 指数退避
    Exponential { multiplier: f64 },
    /// 线性增长
    Linear { increment: u64 },
}

/// 超时配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// 超时时间（毫秒）
    pub duration: u64,
    /// 超时处理动作
    pub on_timeout: Option<String>,
}

/// 计划元数据
#[derive(Debug, Clone)]
pub struct PlanMetadata {
    /// 计划ID
    pub plan_id: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 工作流名称
    pub workflow_name: String,
    /// 工作流版本
    pub workflow_version: String,
    /// 总节点数
    pub total_nodes: usize,
    /// 总阶段数
    pub total_phases: usize,
}

/// 执行器接口 - 所有执行器都必须实现这个接口
pub trait Executor {
    type Input;
    type Output;
    type Error;

    /// 执行输入并返回结果
    async fn execute(
        &mut self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error>;

    /// 获取执行器状态
    fn status(&self) -> ExecutorStatus;

    /// 停止执行器
    async fn stop(&mut self) -> Result<(), Self::Error>;
}

/// 执行器状态
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorStatus {
    /// 空闲状态
    Idle,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// 配置解析器接口
pub trait ConfigParser<T> {
    type Output;
    type Error;

    /// 解析配置
    fn parse(&self, config: T) -> Result<Self::Output, Self::Error>;
}

/// 流程编排器接口
pub trait FlowPlanner {
    type Input;
    type Output;
    type Error;

    /// 创建执行计划
    fn create_execution_plan(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error>;

    /// 优化执行计划
    fn optimize_plan(
        &self,
        plan: Self::Output,
    ) -> Result<Self::Output, Self::Error>;
}

/// 表达式评估器接口
pub trait ExpressionEvaluator {
    type Value;
    type Error;

    /// 评估表达式
    fn evaluate(&self, expression: &str) -> Result<Self::Value, Self::Error>;

    /// 评估条件
    fn evaluate_condition(&self, condition: &str) -> Result<bool, Self::Error>;

    /// 设置变量
    fn set_variable(&mut self, name: String, value: Self::Value);

    /// 获取变量
    fn get_variable(&self, name: &str) -> Option<Self::Value>;
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(
        workflow_name: String,
        workflow_version: String,
        env_vars: HashMap<String, serde_yaml::Value>,
        flow_vars: HashMap<String, serde_yaml::Value>,
    ) -> Self {
        let metadata = PlanMetadata {
            plan_id: uuid::Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now(),
            workflow_name,
            workflow_version,
            total_nodes: 0,
            total_phases: 0,
        };

        Self {
            phases: Vec::new(),
            env_vars,
            flow_vars,
            metadata,
        }
    }

    /// 添加执行阶段
    pub fn add_phase(&mut self, phase: ExecutionPhase) {
        self.metadata.total_nodes += phase.nodes.len();
        self.phases.push(phase);
        self.metadata.total_phases = self.phases.len();
    }

    /// 获取总执行时间估计
    pub fn estimated_duration(&self) -> std::time::Duration {
        // 简化的估计逻辑
        let total_nodes = self.metadata.total_nodes;
        std::time::Duration::from_millis((total_nodes * 100) as u64)
    }

    /// 验证计划的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.phases.is_empty() {
            return Err("执行计划不能为空".to_string());
        }

        // 验证每个阶段
        for phase in &self.phases {
            if phase.nodes.is_empty() {
                return Err(format!("阶段 {} 不能为空", phase.name));
            }
        }

        Ok(())
    }
}

impl ExecutionNode {
    /// 创建新的执行节点
    pub fn new(id: String, name: String, action_spec: ActionSpec) -> Self {
        Self {
            id,
            name,
            node_type: NodeType::Action,
            action_spec,
            dependencies: Vec::new(),
            condition: None,
            priority: 100,
            retry_config: None,
            timeout_config: None,
        }
    }

    /// 添加依赖
    pub fn add_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// 设置条件
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置重试配置
    pub fn with_retry(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }

    /// 设置超时配置
    pub fn with_timeout(mut self, timeout_config: TimeoutConfig) -> Self {
        self.timeout_config = Some(timeout_config);
        self
    }
}
