//! # Runtime 集成模块
//!
//! 提供 YAML 工作流与 flowbuilder-runtime 的集成功能

use crate::config::{ActionType, TaskDefinition, WorkflowConfig};
use crate::expression::ExpressionEvaluator;

use flowbuilder_context::{FlowContext, SharedContext};
use flowbuilder_runtime::{
    BranchCondition, ErrorRecoveryStrategy, FlowNode, FlowOrchestrator, OrchestratorConfig,
    Priority, ScheduledTask, SchedulerConfig, SchedulingStrategy, TaskScheduler,
};

use anyhow::Result;
use serde_yaml;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// YAML 工作流的 Runtime 集成器
pub struct YamlRuntimeIntegrator {
    config: WorkflowConfig,
    evaluator: ExpressionEvaluator,
}

impl YamlRuntimeIntegrator {
    /// 创建新的 Runtime 集成器
    pub fn new(config: WorkflowConfig) -> Result<Self> {
        let mut evaluator = ExpressionEvaluator::new();
        evaluator.set_env_vars(config.workflow.env.clone());
        evaluator.set_flow_vars(config.workflow.vars.clone());

        Ok(Self { config, evaluator })
    }

    /// 从 YAML 字符串创建集成器
    pub fn from_yaml_str(yaml_content: &str) -> Result<Self> {
        let config: WorkflowConfig = serde_yaml::from_str(yaml_content)?;
        Self::new(config)
    }

    /// 从配置创建调度器
    pub fn create_scheduler_from_config(&self) -> Result<TaskScheduler> {
        let config = SchedulerConfig {
            max_concurrent_tasks: self.determine_max_concurrent_tasks(),
            strategy: self.determine_scheduling_strategy(),
            task_timeout: Some(Duration::from_secs(300)),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            enable_dependency_check: true,
        };

        Ok(TaskScheduler::new(config))
    }

    /// 从配置创建编排器
    pub fn create_orchestrator_from_config(&self) -> Result<FlowOrchestrator> {
        let config = OrchestratorConfig {
            max_parallelism: self.determine_max_concurrent_tasks(),
            global_timeout: Some(Duration::from_secs(3600)),
            enable_checkpoints: true,
            checkpoint_interval: Duration::from_secs(60),
            verbose_logging: self.should_enable_verbose_logging(),
        };

        Ok(FlowOrchestrator::with_config(config))
    }

    /// 将 YAML 任务转换为调度任务
    pub fn convert_to_scheduled_tasks(&self) -> Result<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();

        for task_wrapper in &self.config.workflow.tasks {
            let task = &task_wrapper.task;
            let priority = self.determine_task_priority(task)?;
            let dependencies = self.resolve_task_dependencies(task)?;

            let _task_id = task.id.clone();
            let task_name = task.name.clone();
            let actions = task.actions.clone();
            let evaluator = Arc::new(self.evaluator.clone());

            let scheduled_task = ScheduledTask {
                id: Uuid::new_v4(),
                name: task_name,
                description: Some(task.description.clone()),
                priority,
                estimated_duration: self.estimate_task_duration(task),
                created_at: std::time::Instant::now(),
                started_at: None,
                completed_at: None,
                status: flowbuilder_runtime::TaskStatus::Pending,
                dependencies,
                task_fn: Arc::new(move || {
                    Self::execute_task_actions_sync(&actions, (*evaluator).clone())
                }),
                metadata: self.create_task_metadata(task),
            };

            tasks.push(scheduled_task);
        }

        Ok(tasks)
    }

    /// 将 YAML 任务转换为流程节点
    pub fn convert_to_flow_nodes(&self) -> Result<Vec<FlowNode>> {
        let mut nodes = Vec::new();

        for task_wrapper in &self.config.workflow.tasks {
            let task = &task_wrapper.task;

            let node = FlowNode {
                id: task.id.clone(),
                name: task.name.clone(),
                description: Some(task.description.clone()),
                flow: None, // YAML 任务不直接对应 Flow
                condition: self.create_branch_condition(task)?,
                next_nodes: self.determine_next_nodes(task),
                error_recovery: self.determine_error_recovery_strategy(task),
                timeout: self.determine_task_timeout(task),
                retry_config: self.create_retry_config(task),
            };

            nodes.push(node);
        }

        Ok(nodes)
    }

    /// 执行带调度的工作流
    pub async fn execute_with_scheduling(
        &self,
        _context: SharedContext,
    ) -> Result<HashMap<Uuid, flowbuilder_runtime::TaskStatus>> {
        let scheduler = self.create_scheduler_from_config()?;
        let tasks = self.convert_to_scheduled_tasks()?;

        println!("开始调度执行 {} 个任务", tasks.len());

        let mut task_ids = Vec::new();
        for task in tasks {
            let task_id = scheduler.submit_task(task).await?;
            task_ids.push(task_id);
        }

        // 模拟调度执行
        for _ in 0..10 {
            if let Some(task) = scheduler.get_next_task().await {
                if scheduler.can_schedule_task(&task).await {
                    scheduler.execute_task(task).await?;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 收集任务状态
        let mut results = HashMap::new();
        for task_id in task_ids {
            if let Some(status) = scheduler.get_task_status(task_id).await {
                results.insert(task_id, status);
            }
        }

        println!("调度执行完成，处理了 {} 个任务", results.len());
        Ok(results)
    }

    /// 执行带编排的工作流
    pub async fn execute_with_orchestration(
        &self,
        _context: SharedContext,
    ) -> Result<HashMap<String, FlowContext>> {
        let mut orchestrator = self.create_orchestrator_from_config()?;
        let nodes = self.convert_to_flow_nodes()?;

        println!("开始编排执行 {} 个节点", nodes.len());

        for node in nodes {
            orchestrator.add_node(node);
        }

        // 添加依赖关系
        self.setup_orchestrator_dependencies(&mut orchestrator)?;

        let results = orchestrator.execute_all().await?;

        println!("编排执行完成，处理了 {} 个节点", results.len());
        Ok(results)
    }

    /// 混合执行模式：使用调度器和编排器
    pub async fn execute_hybrid(
        &self,
        context: SharedContext,
    ) -> Result<(
        HashMap<Uuid, flowbuilder_runtime::TaskStatus>,
        HashMap<String, FlowContext>,
    )> {
        println!("开始混合模式执行工作流");

        // 并行执行调度和编排
        let context_clone = context.clone();
        let scheduling_future = self.execute_with_scheduling(context);
        let orchestration_future = self.execute_with_orchestration(context_clone);

        let (scheduling_results, orchestration_results) =
            tokio::join!(scheduling_future, orchestration_future);

        println!("混合模式执行完成");
        Ok((scheduling_results?, orchestration_results?))
    }

    // 辅助方法

    /// 确定最大并发任务数
    fn determine_max_concurrent_tasks(&self) -> usize {
        // 可以从环境变量或配置中读取
        std::env::var("FLOWBUILDER_MAX_CONCURRENT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10)
    }

    /// 确定调度策略
    fn determine_scheduling_strategy(&self) -> SchedulingStrategy {
        match self.config.workflow.env.get("FLOWBUILDER_STRATEGY") {
            Some(strategy) if strategy == "fifo" => SchedulingStrategy::FirstInFirstOut,
            Some(strategy) if strategy == "round_robin" => SchedulingStrategy::RoundRobin,
            Some(strategy) if strategy == "shortest_job" => SchedulingStrategy::ShortestJobFirst,
            _ => SchedulingStrategy::Priority,
        }
    }

    /// 确定任务优先级
    fn determine_task_priority(&self, task: &TaskDefinition) -> Result<Priority> {
        // 可以从任务元数据或名称判断优先级
        if task.name.to_lowercase().contains("critical")
            || task.name.to_lowercase().contains("urgent")
        {
            Ok(Priority::Critical)
        } else if task.name.to_lowercase().contains("high") {
            Ok(Priority::High)
        } else if task.name.to_lowercase().contains("low") {
            Ok(Priority::Low)
        } else {
            Ok(Priority::Normal)
        }
    }

    /// 解析任务依赖
    fn resolve_task_dependencies(&self, _task: &TaskDefinition) -> Result<Vec<Uuid>> {
        // TODO: 实现任务依赖解析逻辑
        // 这里需要解析任务配置中的依赖关系
        Ok(Vec::new())
    }

    /// 估算任务执行时间
    fn estimate_task_duration(&self, task: &TaskDefinition) -> Option<Duration> {
        // 基于动作数量和类型估算
        let action_count = task.actions.len();
        let estimated_ms = action_count * 100; // 每个动作估算100ms
        Some(Duration::from_millis(estimated_ms as u64))
    }

    /// 创建任务元数据
    fn create_task_metadata(&self, task: &TaskDefinition) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("task_id".to_string(), task.id.clone());
        metadata.insert("task_name".to_string(), task.name.clone());
        metadata.insert("description".to_string(), task.description.clone());
        metadata.insert("action_count".to_string(), task.actions.len().to_string());
        metadata
    }

    /// 同步执行任务动作
    fn execute_task_actions_sync(
        actions: &[crate::config::Action],
        _evaluator: ExpressionEvaluator,
    ) -> Result<()> {
        for action in actions {
            let action_def = &action.action;
            println!("    执行动作: {} - {}", action_def.id, action_def.name);

            // 简化的动作执行逻辑
            match action_def.action_type {
                ActionType::Builtin => {
                    println!("      内置动作执行完成");
                }
                ActionType::Cmd => {
                    println!("      命令动作执行完成");
                }
                ActionType::Http => {
                    std::thread::sleep(Duration::from_millis(100));
                    println!("      HTTP动作执行完成");
                }
                ActionType::Wasm => {
                    std::thread::sleep(Duration::from_millis(50));
                    println!("      WASM动作执行完成");
                }
            }
        }
        Ok(())
    }

    /// 创建分支条件
    fn create_branch_condition(&self, _task: &TaskDefinition) -> Result<Option<BranchCondition>> {
        // TODO: 从任务配置中解析条件
        Ok(None)
    }

    /// 确定下一个节点
    fn determine_next_nodes(&self, _task: &TaskDefinition) -> Vec<String> {
        // TODO: 从任务配置中解析下一个节点
        Vec::new()
    }

    /// 确定错误恢复策略
    fn determine_error_recovery_strategy(&self, _task: &TaskDefinition) -> ErrorRecoveryStrategy {
        // 可以从任务配置中读取
        ErrorRecoveryStrategy::FailFast
    }

    /// 确定任务超时
    fn determine_task_timeout(&self, _task: &TaskDefinition) -> Option<Duration> {
        // 可以从任务配置中读取
        Some(Duration::from_secs(300))
    }

    /// 创建重试配置
    fn create_retry_config(
        &self,
        _task: &TaskDefinition,
    ) -> Option<flowbuilder_runtime::RetryConfig> {
        // TODO: 从任务配置中解析重试设置
        None
    }

    /// 设置编排器依赖关系
    fn setup_orchestrator_dependencies(&self, _orchestrator: &mut FlowOrchestrator) -> Result<()> {
        // TODO: 根据任务配置设置依赖关系
        Ok(())
    }

    /// 是否启用详细日志
    fn should_enable_verbose_logging(&self) -> bool {
        self.config
            .workflow
            .env
            .get("FLOWBUILDER_VERBOSE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }
}

/// 便捷函数模块，提供简化的 API
pub mod convenience {
    use super::*;
    use flowbuilder_context::{FlowContext, SharedContext};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// 创建默认的流程上下文
    pub fn create_flow_context() -> SharedContext {
        Arc::new(Mutex::new(FlowContext::new_with_trace_id(
            "yaml_runtime_integration".to_string(),
        )))
    }

    /// 使用调度器执行 YAML 工作流
    pub async fn execute_yaml_workflow_with_scheduling(
        yaml_content: &str,
        context: SharedContext,
    ) -> Result<HashMap<Uuid, flowbuilder_runtime::TaskStatus>> {
        let integrator = YamlRuntimeIntegrator::from_yaml_str(yaml_content)?;
        integrator.execute_with_scheduling(context).await
    }

    /// 使用编排器执行 YAML 工作流
    pub async fn execute_yaml_workflow_with_orchestration(
        yaml_content: &str,
        context: SharedContext,
    ) -> Result<HashMap<String, FlowContext>> {
        let integrator = YamlRuntimeIntegrator::from_yaml_str(yaml_content)?;
        integrator.execute_with_orchestration(context).await
    }

    /// 混合执行模式
    pub async fn execute_yaml_workflow_mixed(yaml_content: &str) -> Result<FlowContext> {
        let integrator = YamlRuntimeIntegrator::from_yaml_str(yaml_content)?;
        let context = create_flow_context();

        // 先调度执行
        let _schedule_results = integrator.execute_with_scheduling(context.clone()).await?;

        // 再编排执行
        let _orchestration_results = integrator
            .execute_with_orchestration(context.clone())
            .await?;

        // 返回最终的上下文
        let final_context = context.lock().await.clone();
        Ok(final_context)
    }

    /// 快速调度执行
    pub async fn quick_execute_with_scheduler(
        config: WorkflowConfig,
        context: SharedContext,
    ) -> Result<HashMap<Uuid, flowbuilder_runtime::TaskStatus>> {
        let integrator = YamlRuntimeIntegrator::new(config)?;
        integrator.execute_with_scheduling(context).await
    }

    /// 快速编排执行
    pub async fn quick_execute_with_orchestrator(
        config: WorkflowConfig,
        context: SharedContext,
    ) -> Result<HashMap<String, FlowContext>> {
        let integrator = YamlRuntimeIntegrator::new(config)?;
        integrator.execute_with_orchestration(context).await
    }

    /// 快速混合执行
    pub async fn quick_execute_hybrid(
        config: WorkflowConfig,
        context: SharedContext,
    ) -> Result<FlowContext> {
        let integrator = YamlRuntimeIntegrator::new(config)?;

        // 执行调度
        let _schedule_results = integrator.execute_with_scheduling(context.clone()).await?;

        // 执行编排
        let _orchestration_results = integrator
            .execute_with_orchestration(context.clone())
            .await?;

        // 返回最终上下文
        let final_context = context.lock().await.clone();
        Ok(final_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::WorkflowLoader;

    #[tokio::test]
    async fn test_yaml_runtime_integration() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    FLOWBUILDER_ENV: "test"
    FLOWBUILDER_MAX_CONCURRENT: "5"
  vars:
    name: "Test Workflow"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions:
          - action:
              id: "test_action"
              name: "Test Action"
              description: "A test action"
              type: "builtin"
              outputs:
                status: 200
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let integrator = YamlRuntimeIntegrator::new(config).unwrap();

        // 测试调度器创建
        let scheduler = integrator.create_scheduler_from_config().unwrap();
        assert!(scheduler.get_stats().await.total_scheduled == 0);

        // 测试编排器创建
        let orchestrator = integrator.create_orchestrator_from_config().unwrap();
        assert!(orchestrator.get_all_node_states().await.is_empty());

        // 测试任务转换
        let tasks = integrator.convert_to_scheduled_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Test Task");

        // 测试节点转换
        let nodes = integrator.convert_to_flow_nodes().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "task1");
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions:
          - action:
              id: "test_action"
              name: "Test Action"
              description: "A test action"
              type: "builtin"
              outputs:
                status: 200
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let context = Arc::new(tokio::sync::Mutex::new(FlowContext::new_with_trace_id(
            "test".to_string(),
        )));

        // 测试快速调度执行
        let result =
            convenience::quick_execute_with_scheduler(config.clone(), context.clone()).await;
        assert!(result.is_ok());

        // 测试快速编排执行
        let result =
            convenience::quick_execute_with_orchestrator(config.clone(), context.clone()).await;
        assert!(result.is_ok());

        // 测试快速混合执行
        let result = convenience::quick_execute_hybrid(config, context).await;
        assert!(result.is_ok());
    }
}
