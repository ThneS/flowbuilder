//! # FlowBuilder Runtime - 增强的任务执行器
//!
//! 基于执行计划的任务执行器，负责执行具体的任务

use anyhow::Result;
use flowbuilder_context::SharedContext;
use flowbuilder_core::{
    ActionSpec, ExecutionNode, ExecutionPhase, ExecutionPlan, Executor,
    ExecutorStatus, PhaseExecutionMode, RetryStrategy,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
// tracing 宏无需显式 use 引入

/// 增强的任务执行器
pub struct EnhancedTaskExecutor {
    /// 执行器配置
    config: ExecutorConfig,
    /// 执行器状态
    status: ExecutorStatus,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 执行统计（可选）
    stats: ExecutionStats,
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 默认超时时间（毫秒）
    pub default_timeout: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_timeout: 30000, // 30秒
        }
    }
}

/// 执行统计
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 成功任务数
    pub successful_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 跳过任务数
    pub skipped_tasks: usize,
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 平均执行时间
    pub average_execution_time: Duration,
}

impl Default for EnhancedTaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedTaskExecutor {
    /// 创建新的任务执行器
    pub fn new() -> Self {
        let config = ExecutorConfig::default();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            config,
            status: ExecutorStatus::Idle,
            semaphore,
            stats: ExecutionStats::default(),
        }
    }

    /// 使用配置创建任务执行器
    pub fn with_config(config: ExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            config,
            status: ExecutorStatus::Idle,
            semaphore,
            stats: ExecutionStats::default(),
        }
    }

    /// 执行执行计划
    #[tracing::instrument(level = "info", skip(self, context), fields(workflow = %plan.metadata.workflow_name, phases = plan.phases.len()))]
    pub async fn execute_plan(
        &mut self,
        plan: ExecutionPlan,
        context: SharedContext,
    ) -> Result<ExecutionResult> {
        self.status = ExecutorStatus::Running;
        let start_time = Instant::now();

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(workflow = %plan.metadata.workflow_name, phases = plan.phases.len(), total_nodes = plan.metadata.total_nodes, "开始执行计划");
        }

        let mut result = ExecutionResult {
            plan_id: plan.metadata.plan_id.clone(),
            start_time,
            end_time: None,
            phase_results: Vec::new(),
            total_duration: Duration::default(),
            success: true,
            error_message: None,
        };

        // 设置环境变量和流程变量到上下文
        self.setup_context(&plan, context.clone()).await?;

        // 按阶段执行
        #[cfg(feature = "detailed-logging")]
        for (index, phase) in plan.phases.iter().enumerate() {
            tracing::info!(phase_index = index + 1, phase_name = %phase.name, mode = ?phase.execution_mode, "执行阶段");
            let phase_start = Instant::now();
            let phase_result =
                match self.execute_phase(phase, context.clone()).await {
                    Ok(r) => r,
                    Err(e) => {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        PhaseResult {
                            phase_id: phase.id.clone(),
                            phase_name: phase.name.clone(),
                            start_time: phase_start,
                            end_time: Some(Instant::now()),
                            duration: phase_start.elapsed(),
                            success: false,
                            error_message: Some(e.to_string()),
                            node_results: Vec::new(),
                        }
                    }
                };
            result.phase_results.push(phase_result);
            if !result.success {
                break;
            }
        }
        #[cfg(not(feature = "detailed-logging"))]
        for phase in plan.phases.iter() {
            let phase_start = Instant::now();
            let phase_result =
                match self.execute_phase(phase, context.clone()).await {
                    Ok(r) => r,
                    Err(e) => {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        PhaseResult {
                            phase_id: phase.id.clone(),
                            phase_name: phase.name.clone(),
                            start_time: phase_start,
                            end_time: Some(Instant::now()),
                            duration: phase_start.elapsed(),
                            success: false,
                            error_message: Some(e.to_string()),
                            node_results: Vec::new(),
                        }
                    }
                };
            result.phase_results.push(phase_result);
            if !result.success {
                break;
            }
        }

        result.end_time = Some(Instant::now());
        result.total_duration = start_time.elapsed();

        // 更新统计信息（perf-metrics 特性）
        #[cfg(feature = "perf-metrics")]
        {
            self.update_stats(&result);
        }

        self.status = ExecutorStatus::Idle;

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(total_duration_ms = ?result.total_duration, "执行计划完成");
        }

        Ok(result)
    }

    /// 执行阶段
    #[tracing::instrument(level = "info", skip(self, context), fields(phase = %phase.name, mode = ?phase.execution_mode))]
    async fn execute_phase(
        &mut self,
        phase: &ExecutionPhase,
        context: SharedContext,
    ) -> Result<PhaseResult> {
        let start_time = Instant::now();
        let mut phase_result = PhaseResult {
            phase_id: phase.id.clone(),
            phase_name: phase.name.clone(),
            start_time,
            end_time: None,
            duration: Duration::default(),
            success: true,
            error_message: None,
            node_results: Vec::new(),
        };

        // 检查阶段条件
        if let Some(_condition) = &phase.condition {
            // 这里应该使用表达式评估器检查条件
            // 为了简化，这里假设条件总是满足
            #[cfg(feature = "detailed-logging")]
            {
                tracing::debug!("检查阶段条件(已省略表达式)");
            }
        }

        match phase.execution_mode {
            PhaseExecutionMode::Sequential => {
                for node in &phase.nodes {
                    let node_result =
                        self.execute_node(node, context.clone()).await?;
                    phase_result.node_results.push(node_result);
                }
            }
            PhaseExecutionMode::Parallel => {
                #[cfg(not(feature = "parallel"))]
                {
                    // 并行被禁用时退化为顺序
                    for node in &phase.nodes {
                        let node_result =
                            self.execute_node(node, context.clone()).await?;
                        phase_result.node_results.push(node_result);
                    }
                    phase_result.end_time = Some(Instant::now());
                    phase_result.duration = start_time.elapsed();
                    return Ok(phase_result);
                }

                #[cfg(feature = "parallel")]
                let mut handles = Vec::new();

                for node in &phase.nodes {
                    let node_clone = node.clone();
                    let context_clone = context.clone();
                    let semaphore = self.semaphore.clone();
                    let config = self.config.clone();

                    let handle = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        Self::execute_node_static(
                            &node_clone,
                            context_clone,
                            &config,
                        )
                        .await
                    });

                    handles.push(handle);
                }

                // 等待所有任务完成
                for handle in handles {
                    match handle.await {
                        Ok(node_result) => match node_result {
                            Ok(result) => {
                                phase_result.node_results.push(result)
                            }
                            Err(e) => {
                                phase_result.success = false;
                                phase_result.error_message =
                                    Some(e.to_string());
                                return Err(e);
                            }
                        },
                        Err(e) => {
                            phase_result.success = false;
                            phase_result.error_message = Some(e.to_string());
                            return Err(anyhow::anyhow!("任务执行失败: {}", e));
                        }
                    }
                }
            }
            PhaseExecutionMode::Conditional { condition: _ } => {
                // 检查条件
                #[cfg(feature = "detailed-logging")]
                tracing::debug!("检查条件(已忽略具体表达式)");

                // 简化的条件检查，实际应该使用表达式评估器
                let condition_met = true; // 假设条件满足

                if condition_met {
                    for node in &phase.nodes {
                        let node_result =
                            self.execute_node(node, context.clone()).await?;
                        phase_result.node_results.push(node_result);
                    }
                } else {
                    #[cfg(feature = "detailed-logging")]
                    tracing::info!(phase = %phase.name, "跳过阶段 (条件不满足)");
                }
            }
        }

        phase_result.end_time = Some(Instant::now());
        phase_result.duration = start_time.elapsed();

        Ok(phase_result)
    }

    /// 执行节点
    #[tracing::instrument(level = "debug", skip(self, context), fields(node_id = %node.id, node_name = %node.name))]
    async fn execute_node(
        &mut self,
        node: &ExecutionNode,
        context: SharedContext,
    ) -> Result<NodeResult> {
        Self::execute_node_static(node, context, &self.config).await
    }

    /// 静态执行节点（用于并发执行）
    #[tracing::instrument(level = "debug", skip(context, config), fields(node_id = %node.id, node_name = %node.name))]
    async fn execute_node_static(
        node: &ExecutionNode,
        context: SharedContext,
        config: &ExecutorConfig,
    ) -> Result<NodeResult> {
        let start_time = Instant::now();
        let mut result = NodeResult {
            node_id: node.id.clone(),
            node_name: node.name.clone(),
            start_time,
            end_time: None,
            duration: Duration::default(),
            success: true,
            error_message: None,
            retry_count: 0,
        };

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(node_id = %node.id, node_name = %node.name, "执行节点");
        }

        // 检查节点条件
        if let Some(_condition) = &node.condition {
            #[cfg(feature = "detailed-logging")]
            {
                tracing::debug!("检查节点条件(已省略表达式)");
            }
            // 简化的条件检查
            let condition_met = true;
            if !condition_met {
                #[cfg(feature = "detailed-logging")]
                {
                    tracing::info!(node = %node.name, "跳过节点 (条件不满足)");
                }
                result.end_time = Some(Instant::now());
                result.duration = start_time.elapsed();
                return Ok(result);
            }
        }

        // 执行重试逻辑（可关闭）
        #[cfg(feature = "retry")]
        let max_retries = node
            .retry_config
            .as_ref()
            .map(|c| c.max_retries)
            .unwrap_or(0);
        #[cfg(not(feature = "retry"))]
        let max_retries = 0u32;
        let mut retries = 0;

        loop {
            let execute_result =
                Self::execute_node_action(node, context.clone(), config).await;

            match execute_result {
                Ok(()) => {
                    result.success = true;
                    break;
                }
                Err(e) => {
                    if retries < max_retries {
                        retries += 1;
                        result.retry_count = retries;

                        #[cfg(feature = "detailed-logging")]
                        {
                            tracing::warn!(node = %node.name, retries = retries, max_retries = max_retries, "重试节点");
                        }

                        if let Some(retry_config) = &node.retry_config {
                            #[cfg(not(feature = "retry"))]
                            { /* 重试功能关闭时不进入延迟逻辑 */ }
                            #[cfg(feature = "retry")]
                            let delay = match retry_config.strategy {
                                RetryStrategy::Fixed => retry_config.delay,
                                RetryStrategy::Exponential { multiplier } => {
                                    (retry_config.delay as f64
                                        * multiplier.powi(retries as i32))
                                        as u64
                                }
                                RetryStrategy::Linear { increment } => {
                                    retry_config.delay
                                        + (increment * retries as u64)
                                }
                            };
                            #[cfg(feature = "retry")]
                            tokio::time::sleep(Duration::from_millis(delay))
                                .await;
                        }
                        continue;
                    } else {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        result.end_time = Some(Instant::now());
        result.duration = start_time.elapsed();

        Ok(result)
    }

    /// 执行节点动作
    #[tracing::instrument(level = "debug", skip(context, config), fields(node_id = %node.id, node_name = %node.name, action_type = %node.action_spec.action_type))]
    async fn execute_node_action(
        node: &ExecutionNode,
        context: SharedContext,
        config: &ExecutorConfig,
    ) -> Result<()> {
        let action_spec = &node.action_spec;

        // 设置超时
        let timeout_duration = node
            .timeout_config
            .as_ref()
            .map(|c| Duration::from_millis(c.duration))
            .unwrap_or_else(|| Duration::from_millis(config.default_timeout));

        let action_future = Self::execute_action_by_type(action_spec, context);

        match tokio::time::timeout(timeout_duration, action_future).await {
            Ok(result) => result,
            Err(_) => {
                #[cfg(feature = "detailed-logging")]
                {
                    tracing::error!(node = %node.name, "节点执行超时");
                }
                Err(anyhow::anyhow!("节点 {} 执行超时", node.name))
            }
        }
    }

    /// 根据动作类型执行动作
    async fn execute_action_by_type(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        match action_spec.action_type.as_str() {
            "builtin" => {
                // 执行内置动作
                tokio::time::sleep(Duration::from_millis(100)).await;
                tracing::debug!("执行内置动作");
            }
            "cmd" => {
                // 执行命令动作
                tokio::time::sleep(Duration::from_millis(200)).await;
                tracing::debug!("执行命令动作");
            }
            "http" => {
                // 执行HTTP动作
                tokio::time::sleep(Duration::from_millis(300)).await;
                tracing::debug!("执行HTTP动作");
            }
            "wasm" => {
                // 执行WASM动作
                tokio::time::sleep(Duration::from_millis(150)).await;
                tracing::debug!("执行WASM动作");
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "不支持的动作类型: {}",
                    action_spec.action_type
                ));
            }
        }

        // 存储输出到上下文
        for (key, value) in &action_spec.outputs {
            let mut guard = context.lock().await;
            guard.set_variable(key.clone(), format!("{value:?}"));
        }

        Ok(())
    }

    /// 设置上下文
    #[tracing::instrument(level = "debug", skip(self, context), fields(workflow = %plan.metadata.workflow_name))]
    async fn setup_context(
        &self,
        plan: &ExecutionPlan,
        context: SharedContext,
    ) -> Result<()> {
        let mut guard = context.lock().await;

        // 设置环境变量
        for (key, value) in &plan.env_vars {
            guard.set_variable(format!("env.{key}"), format!("{value:?}"));
        }

        // 设置流程变量
        for (key, value) in &plan.flow_vars {
            guard.set_variable(format!("flow.{key}"), format!("{value:?}"));
        }

        Ok(())
    }

    /// 更新统计信息
    #[cfg(feature = "perf-metrics")]
    fn update_stats(&mut self, result: &ExecutionResult) {
        self.stats.total_execution_time = result.total_duration;

        for phase_result in &result.phase_results {
            for node_result in &phase_result.node_results {
                self.stats.total_tasks += 1;
                if node_result.success {
                    self.stats.successful_tasks += 1;
                } else {
                    self.stats.failed_tasks += 1;
                }
            }
        }

        if self.stats.total_tasks > 0 {
            self.stats.average_execution_time = Duration::from_nanos(
                self.stats.total_execution_time.as_nanos() as u64
                    / self.stats.total_tasks as u64,
            );
        }
    }

    /// 获取执行统计
    #[cfg(feature = "perf-metrics")]
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.stats
    }
}

impl Executor for EnhancedTaskExecutor {
    type Input = (ExecutionPlan, SharedContext);
    type Output = ExecutionResult;
    type Error = anyhow::Error;

    async fn execute(
        &mut self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (plan, context) = input;
        self.execute_plan(plan, context).await
    }

    fn status(&self) -> ExecutorStatus {
        self.status.clone()
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.status = ExecutorStatus::Stopped;
        Ok(())
    }
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 计划ID
    pub plan_id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 阶段结果
    pub phase_results: Vec<PhaseResult>,
    /// 总执行时间
    pub total_duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 阶段结果
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// 阶段ID
    pub phase_id: String,
    /// 阶段名称
    pub phase_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 节点结果
    pub node_results: Vec<NodeResult>,
}

/// 节点结果
#[derive(Debug, Clone)]
pub struct NodeResult {
    /// 节点ID
    pub node_id: String,
    /// 节点名称
    pub node_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 重试次数
    pub retry_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowbuilder_core::{ActionSpec, ExecutionNode};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = EnhancedTaskExecutor::new();
        assert_eq!(executor.status(), ExecutorStatus::Idle);
    }

    #[tokio::test]
    async fn test_node_execution() {
        let config = ExecutorConfig::default();
        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let node = ExecutionNode::new(
            "test_node".to_string(),
            "Test Node".to_string(),
            ActionSpec {
                action_type: "builtin".to_string(),
                parameters: HashMap::new(),
                outputs: HashMap::new(),
            },
        );

        let result =
            EnhancedTaskExecutor::execute_node_static(&node, context, &config)
                .await;
        assert!(result.is_ok());

        let node_result = result.unwrap();
        assert!(node_result.success);
        assert_eq!(node_result.node_id, "test_node");
    }
}
