use crate::config::{ActionDefinition, ActionType, WorkflowConfig};
use crate::expression::ExpressionEvaluator;
use anyhow::Result;
use flowbuilder_context::SharedContext;
use flowbuilder_core::{FlowBuilder, Step, StepFuture};
use std::future::Future;
use std::pin::Pin;
#[allow(unused_imports)]
use std::sync::Arc;
use std::time::Duration;

/// YAML 流程构建器，用于从配置动态构建 FlowBuilder
pub struct YamlFlowBuilder {
    config: WorkflowConfig,
    evaluator: ExpressionEvaluator,
}

/// 复杂的步骤闭包类型
type StepClosure = Box<
    dyn FnMut(
            SharedContext,
        )
            -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + 'static,
>;

impl YamlFlowBuilder {
    /// 从配置创建新的 YAML 流程构建器
    pub fn new(config: WorkflowConfig) -> Result<Self> {
        let mut evaluator = ExpressionEvaluator::new();

        // 设置环境变量和流程变量
        evaluator.set_env_vars(config.workflow.env.clone());
        evaluator.set_flow_vars(config.workflow.vars.clone());

        Ok(Self { config, evaluator })
    }

    /// 构建 FlowBuilder 实例
    pub fn build(&self) -> Result<FlowBuilder> {
        let mut flow_builder = FlowBuilder::new();

        // 按顺序添加任务中的动作
        for task in &self.config.workflow.tasks {
            for action in &task.task.actions {
                let step_closure =
                    self.create_step_closure_from_action(&action.action)?;
                flow_builder = flow_builder.step(step_closure);
            }
        }

        Ok(flow_builder)
    }

    /// 从动作定义创建步骤闭包 (兼容 FnMut)
    fn create_step_closure_from_action(
        &self,
        action: &ActionDefinition,
    ) -> Result<StepClosure> {
        let action_clone = action.clone();
        let evaluator_clone = self.evaluator.clone();

        Ok(Box::new(move |ctx: SharedContext| {
            let action = action_clone.clone();
            let evaluator = evaluator_clone.clone();

            let future: Pin<
                Box<dyn Future<Output = Result<()>> + Send + 'static>,
            > = Box::pin(async move {
                match action.action_type {
                    ActionType::Builtin => {
                        println!("执行内置动作: {}", action.id);
                        // 处理输出
                        for (key, value) in action.outputs {
                            let mut guard = ctx.lock().await;
                            guard.set_variable(key, format!("{value:?}"));
                        }
                    }
                    ActionType::Cmd => {
                        println!("执行命令动作: {}", action.id);
                        // 处理参数
                        for (param_name, param) in action.parameters {
                            let evaluated_value = match &param.value {
                                serde_yaml::Value::String(s) => evaluator
                                    .evaluate(s)
                                    .unwrap_or(param.value.clone()),
                                _ => param.value.clone(),
                            };
                            println!(
                                "  参数 {param_name}: {evaluated_value:?}"
                            );
                        }
                    }
                    ActionType::Http => {
                        println!("执行HTTP动作: {}", action.id);
                        // 模拟HTTP请求
                        tokio::time::sleep(std::time::Duration::from_millis(
                            100,
                        ))
                        .await;
                    }
                    ActionType::Wasm => {
                        println!("执行WASM动作: {}", action.id);
                        // 模拟WASM执行
                        tokio::time::sleep(std::time::Duration::from_millis(
                            50,
                        ))
                        .await;
                    }
                }
                Ok(())
            });
            future
        }))
    }

    /// 从动作定义创建步骤
    #[allow(dead_code)]
    fn create_step_from_action(
        &self,
        action: &ActionDefinition,
    ) -> Result<Step> {
        match action.action_type {
            ActionType::Builtin => self.create_builtin_step(action),
            ActionType::Cmd => self.create_cmd_step(action),
            ActionType::Http => self.create_http_step(action),
            ActionType::Wasm => self.create_wasm_step(action),
        }
    }

    /// 创建内置步骤
    fn create_builtin_step(&self, action: &ActionDefinition) -> Result<Step> {
        let action_id = action.id.clone();
        let outputs = action.outputs.clone();
        let evaluator = self.evaluator.clone();

        Ok(Box::new(move |_ctx: SharedContext| -> StepFuture {
            let action_id = action_id.clone();
            let outputs = outputs.clone();
            let mut evaluator = evaluator.clone();

            Box::pin(async move {
                println!("执行内置动作: {action_id}");

                // 将输出存储到上下文中
                for (key, value) in outputs {
                    let full_key = format!("{action_id}.outputs.{key}");
                    evaluator.set_context_var(full_key, value);
                }

                // 模拟一些处理时间
                tokio::time::sleep(Duration::from_millis(100)).await;

                Ok(())
            })
        }))
    }

    /// 创建命令步骤
    fn create_cmd_step(&self, action: &ActionDefinition) -> Result<Step> {
        let action_id = action.id.clone();
        let parameters = action.parameters.clone();
        let outputs = action.outputs.clone();
        let evaluator = self.evaluator.clone();

        Ok(Box::new(move |_ctx: SharedContext| -> StepFuture {
            let action_id = action_id.clone();
            let parameters = parameters.clone();
            let outputs = outputs.clone();
            let mut evaluator = evaluator.clone();

            Box::pin(async move {
                println!("执行命令动作: {action_id}");

                // 处理参数
                for (param_name, param) in parameters {
                    let evaluated_value = match &param.value {
                        serde_yaml::Value::String(s) => {
                            evaluator.evaluate(s).unwrap_or(param.value.clone())
                        }
                        _ => param.value.clone(),
                    };
                    println!("  参数 {param_name}: {evaluated_value:?}");
                }

                // 模拟命令执行
                tokio::time::sleep(Duration::from_millis(200)).await;

                // 将输出存储到上下文中
                for (key, value) in outputs {
                    let full_key = format!("{action_id}.outputs.{key}");
                    evaluator.set_context_var(full_key, value);
                }

                Ok(())
            })
        }))
    }

    /// 创建 HTTP 步骤
    fn create_http_step(&self, action: &ActionDefinition) -> Result<Step> {
        let action_id = action.id.clone();
        let parameters = action.parameters.clone();
        let outputs = action.outputs.clone();
        let evaluator = self.evaluator.clone();

        Ok(Box::new(move |_ctx: SharedContext| -> StepFuture {
            let action_id = action_id.clone();
            let parameters = parameters.clone();
            let outputs = outputs.clone();
            let mut evaluator = evaluator.clone();

            Box::pin(async move {
                println!("执行 HTTP 动作: {action_id}");

                // 处理参数（URL, 方法, 头部等）
                for (param_name, param) in parameters {
                    let evaluated_value = match &param.value {
                        serde_yaml::Value::String(s) => {
                            evaluator.evaluate(s).unwrap_or(param.value.clone())
                        }
                        _ => param.value.clone(),
                    };
                    println!("  HTTP 参数 {param_name}: {evaluated_value:?}");
                }

                // 模拟 HTTP 请求
                tokio::time::sleep(Duration::from_millis(300)).await;

                // 将输出存储到上下文中
                for (key, value) in outputs {
                    let full_key = format!("{action_id}.outputs.{key}");
                    evaluator.set_context_var(full_key, value);
                }

                Ok(())
            })
        }))
    }

    /// 创建 WASM 步骤
    fn create_wasm_step(&self, action: &ActionDefinition) -> Result<Step> {
        let action_id = action.id.clone();
        let parameters = action.parameters.clone();
        let outputs = action.outputs.clone();
        let evaluator = self.evaluator.clone();

        Ok(Box::new(move |_ctx: SharedContext| -> StepFuture {
            let action_id = action_id.clone();
            let parameters = parameters.clone();
            let outputs = outputs.clone();
            let mut evaluator = evaluator.clone();

            Box::pin(async move {
                println!("执行 WASM 动作: {action_id}");

                // 处理参数
                for (param_name, param) in parameters {
                    let evaluated_value = match &param.value {
                        serde_yaml::Value::String(s) => {
                            evaluator.evaluate(s).unwrap_or(param.value.clone())
                        }
                        _ => param.value.clone(),
                    };
                    println!("  WASM 参数 {param_name}: {evaluated_value:?}");
                }

                // 模拟 WASM 执行
                tokio::time::sleep(Duration::from_millis(150)).await;

                // 将输出存储到上下文中
                for (key, value) in outputs {
                    let full_key = format!("{action_id}.outputs.{key}");
                    evaluator.set_context_var(full_key, value);
                }

                Ok(())
            })
        }))
    }

    /// 获取表达式求值器的可变引用
    pub fn evaluator_mut(&mut self) -> &mut ExpressionEvaluator {
        &mut self.evaluator
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

    #[test]
    fn test_yaml_flow_builder() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    FLOWBUILDER_ENV: "test"
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
                message: "Test completed"
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let yaml_builder = YamlFlowBuilder::new(config).unwrap();
        let flow_builder = yaml_builder.build().unwrap();

        // 验证流程构建器创建成功
        // 注意：我们无法直接访问 steps 字段，但可以构建流程
        let _flow = flow_builder.build();
    }

    #[tokio::test]
    async fn test_builtin_step_execution() {
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
        let yaml_builder = YamlFlowBuilder::new(config).unwrap();
        let flow_builder = yaml_builder.build().unwrap();

        // 创建上下文并执行流程
        let _context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let flow = flow_builder.build();
        let result = flow.execute().await;
        assert!(result.is_ok());
    }
}
