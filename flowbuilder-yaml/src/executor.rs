//! # FlowBuilder YAML - 统一的流程执行器
//!
//! 实现新的分层架构：配置解析器 → 流程编排器 → 任务执行器

use crate::config::WorkflowConfig;
use crate::config_parser::YamlConfigParser;
use crate::expression::ExpressionEvaluator;
use anyhow::{Context, Result};
use flowbuilder_context::SharedContext;
#[cfg(feature = "runtime")]
use flowbuilder_core::ExecutionPlan;
use flowbuilder_core::{Executor, ExecutorStatus};
#[cfg(all(feature = "runtime", feature = "perf-metrics"))]
use flowbuilder_runtime::ExecutionStats;
#[cfg(feature = "runtime")]
use flowbuilder_runtime::{
    EnhancedFlowOrchestrator, EnhancedTaskExecutor, ExecutionComplexity,
    ExecutionResult, ExecutorConfig,
};
#[cfg(feature = "runtime")]
use tracing::{debug, info};

#[cfg(not(feature = "runtime"))]
#[derive(Debug, Clone)]
pub struct ExecutionResultPlaceholder;

#[cfg(all(test, feature = "runtime"))]
use std::sync::Arc;

/// 统一的动态流程执行器
pub struct DynamicFlowExecutor {
    /// 原始配置
    config: WorkflowConfig,
    /// 配置解析器
    parser: YamlConfigParser,
    /// 流程编排器
    #[cfg(feature = "runtime")]
    orchestrator: EnhancedFlowOrchestrator,
    /// 任务执行器
    #[cfg(feature = "runtime")]
    executor: EnhancedTaskExecutor,
    /// 表达式评估器
    evaluator: ExpressionEvaluator,
    /// 是否在执行前打印执行计划
    #[cfg(feature = "runtime")]
    print_plan: bool,
}

impl DynamicFlowExecutor {
    /// 创建新的动态流程执行器
    pub fn new(config: WorkflowConfig) -> Result<Self> {
        // 创建配置解析器
        let parser = YamlConfigParser::new(config.clone());

        // 验证配置
        parser.validate().context("配置验证失败")?;

        // 创建流程编排器
        #[cfg(feature = "runtime")]
        let orchestrator = EnhancedFlowOrchestrator::new();
        #[cfg(feature = "runtime")]
        let executor = EnhancedTaskExecutor::new();

        // 创建表达式评估器
        let mut evaluator = ExpressionEvaluator::new();
        evaluator.set_env_vars(config.workflow.env.clone());
        evaluator.set_flow_vars(config.workflow.vars.clone());

        Ok(Self {
            config,
            parser,
            #[cfg(feature = "runtime")]
            orchestrator,
            #[cfg(feature = "runtime")]
            executor,
            evaluator,
            #[cfg(feature = "runtime")]
            print_plan: false,
        })
    }

    /// 使用自定义执行器配置创建
    #[cfg(feature = "runtime")]
    pub fn with_executor_config(
        config: WorkflowConfig,
        executor_config: ExecutorConfig,
    ) -> Result<Self> {
        let parser = YamlConfigParser::new(config.clone());
        parser.validate().context("配置验证失败")?;

        #[cfg(feature = "runtime")]
        let orchestrator = EnhancedFlowOrchestrator::new();
        #[cfg(feature = "runtime")]
        let executor = EnhancedTaskExecutor::with_config(executor_config);

        let mut evaluator = ExpressionEvaluator::new();
        evaluator.set_env_vars(config.workflow.env.clone());
        evaluator.set_flow_vars(config.workflow.vars.clone());

        Ok(Self {
            config,
            parser,
            #[cfg(feature = "runtime")]
            orchestrator,
            #[cfg(feature = "runtime")]
            executor,
            evaluator,
            #[cfg(feature = "runtime")]
            print_plan: false,
        })
    }

    /// 执行工作流 - 新的分层架构实现
    #[cfg(feature = "runtime")]
    pub async fn execute(
        &mut self,
        context: SharedContext,
    ) -> Result<ExecutionResult> {
        info!("开始执行工作流，使用新的分层架构");

        // 第1步：解析配置，生成执行节点
        let parse_result = self.parser.parse_full().context("配置解析失败")?;

        info!("配置解析完成");
        info!(workflow_name = %parse_result.workflow_name, workflow_version = %parse_result.workflow_version, node_count = parse_result.nodes.len());

        // 第2步：流程编排，生成执行计划
        let env_vars = parse_result
            .env_vars
            .into_iter()
            .map(|(k, v)| (k, serde_yaml::Value::String(v)))
            .collect();

        #[cfg(feature = "runtime")]
        let execution_plan = self
            .orchestrator
            .create_execution_plan(
                parse_result.nodes,
                env_vars,
                parse_result.flow_vars,
                parse_result.workflow_name,
                parse_result.workflow_version,
            )
            .context("执行计划创建失败")?;

        #[cfg(not(feature = "runtime"))]
        let execution_plan: ExecutionPlan = {
            return Err(anyhow::anyhow!(
                "运行时未启用: 请开启 feature 'runtime' 后使用执行功能"
            ));
        };

        info!("执行计划生成完成");
        info!(phases = execution_plan.phases.len(), total_nodes = execution_plan.metadata.total_nodes, est_duration_ms = ?execution_plan.estimated_duration());

        // 可选：打印详细执行计划
        if self.print_plan {
            let pretty = execution_plan.to_pretty_string();
            debug!(plan_pretty = %pretty, "执行计划明细");
        }

        // 第3步：分析执行复杂度
        #[cfg(feature = "runtime")]
        let complexity = self.orchestrator.analyze_complexity(&execution_plan);
        #[cfg(feature = "runtime")]
        info!("执行复杂度分析");
        #[cfg(feature = "runtime")]
        {
            info!(
                score = complexity.complexity_score,
                max_parallel = complexity.max_parallel_nodes,
                conditional_nodes = complexity.conditional_nodes
            );
        }

        // 第4步：执行任务
        #[cfg(feature = "runtime")]
        let result = self
            .executor
            .execute_plan(execution_plan, context)
            .await
            .context("任务执行失败")?;

        info!("工作流执行完成");
        info!(success = result.success, total_duration_ms = ?result.total_duration, phases = result.phase_results.len());

        // 打印执行统计
        #[cfg(all(feature = "runtime", feature = "perf-metrics"))]
        {
            let stats = self.executor.get_stats();
            info!("执行统计");
            info!(total_tasks = stats.total_tasks, successful_tasks = stats.successful_tasks, failed_tasks = stats.failed_tasks, average_execution_time_ms = ?stats.average_execution_time);
        }

        Ok(result)
    }

    /// 获取执行计划预览（不执行）
    #[cfg(feature = "runtime")]
    pub fn get_execution_plan_preview(&self) -> Result<ExecutionPlan> {
        let parse_result = self.parser.parse_full().context("配置解析失败")?;

        let env_vars = parse_result
            .env_vars
            .into_iter()
            .map(|(k, v)| (k, serde_yaml::Value::String(v)))
            .collect();

        self.orchestrator.create_execution_plan(
            parse_result.nodes,
            env_vars,
            parse_result.flow_vars,
            parse_result.workflow_name,
            parse_result.workflow_version,
        )
    }

    /// 分析工作流复杂度
    #[cfg(feature = "runtime")]
    pub fn analyze_workflow_complexity(&self) -> Result<ExecutionComplexity> {
        let execution_plan = self.get_execution_plan_preview()?;
        Ok(self.orchestrator.analyze_complexity(&execution_plan))
    }

    /// 生成并返回“可读”的执行计划字符串（不执行）
    #[cfg(feature = "runtime")]
    pub fn print_execution_plan(&self) -> Result<String> {
        let plan = self.get_execution_plan_preview()?;
        Ok(plan.to_pretty_string())
    }

    /// 设置是否打印执行计划
    #[cfg(feature = "runtime")]
    pub fn set_print_plan(&mut self, enabled: bool) {
        self.print_plan = enabled;
    }

    /// 验证工作流配置
    #[cfg(feature = "runtime")]
    pub fn validate_workflow(&self) -> Result<()> {
        // 验证配置
        self.parser.validate()?;

        // 验证执行计划
        let execution_plan = self.get_execution_plan_preview()?;
        execution_plan
            .validate()
            .map_err(|e| anyhow::anyhow!("执行计划验证失败: {}", e))?;

        Ok(())
    }

    /// 获取执行统计信息
    #[cfg(all(feature = "runtime", feature = "perf-metrics"))]
    pub fn get_stats(&self) -> &ExecutionStats {
        self.executor.get_stats()
    }

    /// 获取工作流信息
    pub fn get_workflow_info(&self) -> WorkflowInfo {
        WorkflowInfo {
            name: self.parser.get_workflow_name(),
            version: self.parser.get_workflow_version(),
            task_count: self.config.workflow.tasks.len(),
            env_var_count: self.config.workflow.env.len(),
            flow_var_count: self.config.workflow.vars.len(),
        }
    }

    /// 获取表达式评估器
    pub fn evaluator(&self) -> &ExpressionEvaluator {
        &self.evaluator
    }

    /// 获取工作流配置
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// 获取执行器状态
    #[cfg(feature = "runtime")]
    pub fn executor_status(&self) -> ExecutorStatus {
        self.executor.status()
    }
    #[cfg(not(feature = "runtime"))]
    pub fn executor_status(&self) -> ExecutorStatus {
        ExecutorStatus::Idle
    }

    /// 停止执行器
    #[cfg(feature = "runtime")]
    pub async fn stop(&mut self) -> Result<()> {
        self.executor.stop().await
    }
    #[cfg(not(feature = "runtime"))]
    pub async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 工作流信息
#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    /// 工作流名称
    pub name: String,
    /// 工作流版本
    pub version: String,
    /// 任务数量
    pub task_count: usize,
    /// 环境变量数量
    pub env_var_count: usize,
    /// 流程变量数量
    pub flow_var_count: usize,
}

#[cfg(feature = "runtime")]
impl Executor for DynamicFlowExecutor {
    type Input = SharedContext;
    type Output = ExecutionResult;
    type Error = anyhow::Error;

    async fn execute(
        &mut self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        self.execute(input).await
    }

    fn status(&self) -> ExecutorStatus {
        self.executor_status()
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.stop().await
    }
}

#[cfg(not(feature = "runtime"))]
impl Executor for DynamicFlowExecutor {
    type Input = SharedContext;
    type Output = (); // 无执行逻辑
    type Error = anyhow::Error;

    async fn execute(
        &mut self,
        _input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Err(anyhow::anyhow!(
            "运行时未启用: 启用 feature 'runtime' 才能执行工作流"
        ))
    }

    fn status(&self) -> ExecutorStatus {
        self.executor_status()
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::WorkflowLoader;
    #[cfg(feature = "runtime")]
    use flowbuilder_context::FlowContext;

    #[cfg(feature = "runtime")]
    #[tokio::test]
    async fn test_new_architecture_execution() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    FLOWBUILDER_ENV: "test"
  vars:
    name: "New Architecture Test"
    description: "Testing new layered architecture"
  tasks:
    - task:
        id: "task1"
        name: "Test Task 1"
        description: "First test task"
        actions:
          - action:
              id: "action1"
              name: "Test Action 1"
              description: "First test action"
              type: "builtin"
              flow:
                next: null
              outputs:
                result: "success"
              parameters: {}
    - task:
        id: "task2"
        name: "Test Task 2"
        description: "Second test task"
        actions:
          - action:
              id: "action2"
              name: "Test Action 2"
              description: "Second test action"
              type: "cmd"
              flow:
                next: null
              outputs:
                status: "completed"
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let mut executor = DynamicFlowExecutor::new(config).unwrap();

        let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));
        let result = executor.execute(context).await;

        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.success);
        assert_eq!(execution_result.phase_results.len(), 1); // 两个任务应该在同一阶段
    }

    #[cfg(feature = "runtime")]
    #[tokio::test]
    async fn test_execution_plan_preview() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "preview"
  vars:
    name: "Preview Test"
  tasks:
    - task:
        id: "preview_task"
        name: "Preview Task"
        description: "Task for preview testing"
        actions:
          - action:
              id: "preview_action"
              name: "Preview Action"
              description: "Action for preview"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let executor = DynamicFlowExecutor::new(config).unwrap();

        let plan = executor.get_execution_plan_preview().unwrap();
        assert_eq!(plan.phases.len(), 1);
        assert_eq!(plan.metadata.total_nodes, 1);
        assert_eq!(plan.metadata.workflow_name, "Preview Test");
    }

    #[cfg(feature = "runtime")]
    #[tokio::test]
    async fn test_workflow_complexity_analysis() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "complexity"
  vars:
    name: "Complexity Test"
  tasks:
    - task:
        id: "complex_task"
        name: "Complex Task"
        description: "A complex task for analysis"
        actions:
          - action:
              id: "complex_action"
              name: "Complex Action"
              description: "A complex action"
              type: "builtin"
              flow:
                next_if: "true"
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let executor = DynamicFlowExecutor::new(config).unwrap();

        let complexity = executor.analyze_workflow_complexity().unwrap();
        assert_eq!(complexity.total_nodes, 1);
        assert_eq!(complexity.conditional_nodes, 1);
        assert!(complexity.complexity_score > 0.0);
    }

    #[test]
    fn test_workflow_info() {
        let yaml_content = r#"
workflow:
  version: "2.0"
  env:
    ENV1: "value1"
    ENV2: "value2"
  vars:
    name: "Info Test Workflow"
    var1: "value1"
  tasks:
    - task:
        id: "info_task"
        name: "Info Task"
        description: "Task for info testing"
        actions:
          - action:
              id: "info_action"
              name: "Info Action"
              description: "Action for info"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let executor = DynamicFlowExecutor::new(config).unwrap();

        let info = executor.get_workflow_info();
        assert_eq!(info.name, "Info Test Workflow");
        assert_eq!(info.version, "2.0");
        assert_eq!(info.task_count, 1);
        assert_eq!(info.env_var_count, 2);
        assert_eq!(info.flow_var_count, 2);
    }

    #[cfg(feature = "runtime")]
    #[tokio::test]
    async fn test_execution_plan_dependency_phases() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env: {}
  vars:
    name: "Execution Plan Test"
  tasks:
    - task:
        id: "setup_task"
        name: "Setup Task"
        description: "First task in the chain"
        actions:
          - action:
              id: "setup_action"
              name: "Setup Action"
              description: "Setup action"
              type: "builtin"
              flow:
                next: "notification_task"
              outputs: {}
              parameters: {}
    - task:
        id: "notification_task"
        name: "Notification Task"
        description: "Second task in the chain"
        actions:
          - action:
              id: "notification_action"
              name: "Notification Action"
              description: "Notification action"
              type: "builtin"
              flow:
                next: "process_task"
              outputs: {}
              parameters: {}
    - task:
        id: "process_task"
        name: "Process Task"
        description: "Third task in the chain"
        actions:
          - action:
              id: "process_action"
              name: "Process Action"
              description: "Process action"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let executor = DynamicFlowExecutor::new(config).unwrap();

        // Create execution plan
        let plan = executor.get_execution_plan_preview().unwrap();

        // Verify the execution plan has correct number of phases
        // Each task should be in its own phase due to dependencies
        assert_eq!(
            plan.phases.len(),
            3,
            "Should have 3 phases for dependent tasks"
        );

        // Verify the order of tasks in phases
        // Phase 0: setup_task (no dependencies)
        // Phase 1: notification_task (depends on setup_task)
        // Phase 2: process_task (depends on notification_task)

        assert_eq!(plan.phases[0].nodes.len(), 1, "Phase 0 should have 1 node");
        assert_eq!(
            plan.phases[0].nodes[0].id, "setup_task",
            "Phase 0 should contain setup_task"
        );

        assert_eq!(plan.phases[1].nodes.len(), 1, "Phase 1 should have 1 node");
        assert_eq!(
            plan.phases[1].nodes[0].id, "notification_task",
            "Phase 1 should contain notification_task"
        );

        assert_eq!(plan.phases[2].nodes.len(), 1, "Phase 2 should have 1 node");
        assert_eq!(
            plan.phases[2].nodes[0].id, "process_task",
            "Phase 2 should contain process_task"
        );

        // Verify dependencies are correctly set
        assert_eq!(
            plan.phases[0].nodes[0].dependencies,
            Vec::<String>::new(),
            "setup_task should have no dependencies"
        );
        assert_eq!(
            plan.phases[1].nodes[0].dependencies,
            vec!["setup_task"],
            "notification_task should depend on setup_task"
        );
        assert_eq!(
            plan.phases[2].nodes[0].dependencies,
            vec!["notification_task"],
            "process_task should depend on notification_task"
        );
    }
}
