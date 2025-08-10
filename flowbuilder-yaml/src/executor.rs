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
        })
    }

    /// 执行工作流 - 新的分层架构实现
    #[cfg(feature = "runtime")]
    pub async fn execute(
        &mut self,
        context: SharedContext,
    ) -> Result<ExecutionResult> {
        println!("开始执行工作流，使用新的分层架构");

        // 第1步：解析配置，生成执行节点
        let parse_result = self.parser.parse_full().context("配置解析失败")?;

        println!("配置解析完成：");
        println!("  工作流名称: {}", parse_result.workflow_name);
        println!("  工作流版本: {}", parse_result.workflow_version);
        println!("  节点数量: {}", parse_result.nodes.len());

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

        println!("执行计划生成完成：");
        println!("  总阶段数: {}", execution_plan.phases.len());
        println!("  总节点数: {}", execution_plan.metadata.total_nodes);
        println!("  预计耗时: {:?}", execution_plan.estimated_duration());

        // 第3步：分析执行复杂度
        #[cfg(feature = "runtime")]
        let complexity = self.orchestrator.analyze_complexity(&execution_plan);
        #[cfg(feature = "runtime")]
        println!("执行复杂度分析：");
        #[cfg(feature = "runtime")]
        {
            println!("  复杂度分数: {:.2}", complexity.complexity_score);
            println!("  最大并行度: {}", complexity.max_parallel_nodes);
            println!("  条件节点数: {}", complexity.conditional_nodes);
        }

        // 第4步：执行任务
        #[cfg(feature = "runtime")]
        let result = self
            .executor
            .execute_plan(execution_plan, context)
            .await
            .context("任务执行失败")?;

        println!("工作流执行完成：");
        println!(
            "  执行结果: {}",
            if result.success { "成功" } else { "失败" }
        );
        println!("  总耗时: {:?}", result.total_duration);
        println!("  阶段数: {}", result.phase_results.len());

        // 打印执行统计
        #[cfg(all(feature = "runtime", feature = "perf-metrics"))]
        {
            let stats = self.executor.get_stats();
            println!("执行统计：");
            println!("  总任务数: {}", stats.total_tasks);
            println!("  成功任务数: {}", stats.successful_tasks);
            println!("  失败任务数: {}", stats.failed_tasks);
            println!("  平均执行时间: {:?}", stats.average_execution_time);
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
}
