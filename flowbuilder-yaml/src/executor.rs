use crate::config::{WorkflowConfig, FlowControl};
use crate::parser::YamlFlowBuilder;
use crate::expression::ExpressionEvaluator;
use flowbuilder_core::{FlowBuilder, FlowExecutor};
use flowbuilder_context::{SharedContext, FlowContext};
use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Duration;

/// 动态流程执行器，支持条件流程控制、重试、超时等高级功能
pub struct DynamicFlowExecutor {
    config: WorkflowConfig,
    evaluator: ExpressionEvaluator,
}

impl DynamicFlowExecutor {
    /// 创建新的动态流程执行器
    pub fn new(config: WorkflowConfig) -> Result<Self> {
        let mut evaluator = ExpressionEvaluator::new();

        // 设置环境变量和流程变量
        evaluator.set_env_vars(config.workflow.env.clone());
        evaluator.set_flow_vars(config.workflow.vars.clone());

        Ok(Self {
            config,
            evaluator,
        })
    }

    /// 执行动态流程
    pub async fn execute(&mut self, context: SharedContext) -> Result<()> {
        println!("开始执行动态工作流: {} v{}",
                 self.config.workflow.vars.get("name")
                     .map(|v| v.as_str().unwrap_or("Unknown"))
                     .unwrap_or("Unknown"),
                 self.config.workflow.version);

        // 按顺序执行任务
        for task in &self.config.workflow.tasks.clone() {
            println!("执行任务: {} - {}", task.task.id, task.task.name);

            for action in &task.task.actions {
                self.execute_action(&action.action, context.clone()).await
                    .with_context(|| format!("Failed to execute action: {}", action.action.id))?;
            }
        }

        println!("工作流执行完成");
        Ok(())
    }

    /// 执行单个动作
    async fn execute_action(
        &mut self,
        action: &crate::config::ActionDefinition,
        context: SharedContext,
    ) -> Result<()> {
        let action_id = &action.id;
        let flow_control = &action.flow;

        println!("  执行动作: {} - {}", action_id, action.name);

        // 检查执行条件
        if let Some(next_if) = &flow_control.next_if {
            if !self.evaluator.evaluate_condition(next_if)? {
                println!("    跳过动作 {} (条件不满足)", action_id);
                return Ok(());
            }
        }

        // 执行重试逻辑
        let mut retries = 0;
        let max_retries = flow_control.retry.as_ref()
            .map(|r| r.max_retries)
            .unwrap_or(0);

        loop {
            let result = self.execute_action_with_timeout(action, context.clone()).await;

            match result {
                Ok(()) => {
                    // 执行成功，存储输出
                    self.store_action_outputs(action_id, &action.outputs).await?;

                    // 检查下一步流程
                    self.handle_next_flow(flow_control).await?;
                    break;
                }
                Err(e) => {
                    println!("    动作 {} 执行失败: {}", action_id, e);

                    if retries < max_retries {
                        retries += 1;
                        println!("    重试 {}/{}", retries, max_retries);

                        if let Some(retry_config) = &flow_control.retry {
                            tokio::time::sleep(Duration::from_millis(retry_config.delay)).await;
                        }
                        continue;
                    } else {
                        // 处理错误流程
                        if let Some(on_error) = &flow_control.on_error {
                            self.handle_error_flow(on_error, &e).await?;
                        }
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 带超时的动作执行
    async fn execute_action_with_timeout(
        &self,
        action: &crate::config::ActionDefinition,
        context: SharedContext,
    ) -> Result<()> {
        let action_future = self.execute_raw_action(action, context);

        if let Some(timeout_config) = &action.flow.timeout {
            let timeout_duration = Duration::from_millis(timeout_config.duration);

            match tokio::time::timeout(timeout_duration, action_future).await {
                Ok(result) => result,
                Err(_) => {
                    let timeout_error = anyhow::anyhow!("Action timed out after {}ms", timeout_config.duration);

                    // 处理超时流程
                    if let Some(on_timeout) = &action.flow.on_timeout {
                        self.handle_timeout_flow(on_timeout).await?;
                    }

                    Err(timeout_error)
                }
            }
        } else {
            action_future.await
        }
    }

    /// 执行原始动作
    async fn execute_raw_action(
        &self,
        action: &crate::config::ActionDefinition,
        _context: SharedContext,
    ) -> Result<()> {
        match action.action_type {
            crate::config::ActionType::Builtin => {
                println!("    执行内置动作");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            crate::config::ActionType::Cmd => {
                println!("    执行命令动作");

                // 处理参数
                for (param_name, param) in &action.parameters {
                    let evaluated_value = self.evaluator.evaluate(&format!("{:?}", param.value))
                        .unwrap_or(param.value.clone());
                    println!("      参数 {}: {:?}", param_name, evaluated_value);
                }

                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            crate::config::ActionType::Http => {
                println!("    执行 HTTP 动作");
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            crate::config::ActionType::Wasm => {
                println!("    执行 WASM 动作");
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
        }

        Ok(())
    }

    /// 存储动作输出到上下文
    async fn store_action_outputs(
        &mut self,
        action_id: &str,
        outputs: &std::collections::HashMap<String, serde_yaml::Value>,
    ) -> Result<()> {
        for (key, value) in outputs {
            let full_key = format!("{}.outputs.{}", action_id, key);
            self.evaluator.set_context_var(full_key, value.clone());
            println!("    输出 {}: {:?}", key, value);
        }
        Ok(())
    }

    /// 处理下一步流程
    async fn handle_next_flow(&self, flow_control: &FlowControl) -> Result<()> {
        if let Some(next) = &flow_control.next {
            if next != "null" {
                println!("    下一步: {}", next);
            }
        }

        // 处理循环控制
        if let Some(while_util) = &flow_control.while_util {
            if self.evaluator.evaluate_condition(&while_util.condition)? {
                println!("    循环条件满足，最大迭代次数: {}", while_util.max_iterations);
            }
        }

        Ok(())
    }

    /// 处理错误流程
    async fn handle_error_flow(&self, on_error: &str, error: &anyhow::Error) -> Result<()> {
        println!("    错误处理: {} - {}", on_error, error);
        Ok(())
    }

    /// 处理超时流程
    async fn handle_timeout_flow(&self, on_timeout: &str) -> Result<()> {
        println!("    超时处理: {}", on_timeout);
        Ok(())
    }

    /// 获取表达式求值器的引用
    pub fn evaluator(&self) -> &ExpressionEvaluator {
        &self.evaluator
    }

    /// 获取工作流配置的引用
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::WorkflowLoader;

    #[tokio::test]
    async fn test_dynamic_flow_execution() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    FLOWBUILDER_ENV: "test"
  vars:
    name: "Dynamic Test Workflow"
    description: "Testing dynamic execution"
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
              flow:
                next: null
                retry:
                  max_retries: 2
                  delay: 100
                timeout:
                  duration: 1000
              outputs:
                status: 200
                message: "Test completed"
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let mut executor = DynamicFlowExecutor::new(config).unwrap();

        let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));
        let result = executor.execute(context).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dynamic_flow_with_condition() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "production"
  vars:
    enabled: true
  tasks:
    - task:
        id: "conditional_task"
        name: "Conditional Task"
        description: "A conditional test task"
        actions:
          - action:
              id: "conditional_action"
              name: "Conditional Action"
              description: "An action with condition"
              type: "builtin"
              flow:
                next_if: "true"
                next: null
              outputs:
                result: "executed"
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let mut executor = DynamicFlowExecutor::new(config).unwrap();

        let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));
        let result = executor.execute(context).await;

        assert!(result.is_ok());
    }
}
