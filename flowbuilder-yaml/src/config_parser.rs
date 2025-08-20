//! # FlowBuilder YAML - 配置解析器
//!
//! 从YAML配置解析生成执行节点

use crate::config::{ActionDefinition, TaskDefinition, WorkflowConfig};
use anyhow::Result;
use flowbuilder_core::{
    ActionSpec, ConfigParser, ExecutionNode, NodeType, RetryConfig,
    RetryStrategy, TimeoutConfig,
};
use std::collections::HashMap;

/// YAML配置解析器
pub struct YamlConfigParser {
    config: WorkflowConfig,
}

impl YamlConfigParser {
    /// 创建新的配置解析器
    pub fn new(config: WorkflowConfig) -> Self {
        Self { config }
    }

    /// 解析配置，生成执行节点列表
    pub fn parse(&self) -> Result<Vec<ExecutionNode>> {
        let mut nodes = Vec::new();

        for task_wrapper in &self.config.workflow.tasks {
            let task = &task_wrapper.task;

            // 为每个任务创建执行节点
            let node = self.create_execution_node(task)?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// 创建执行节点
    fn create_execution_node(
        &self,
        task: &TaskDefinition,
    ) -> Result<ExecutionNode> {
        // 合并所有动作为一个节点
        let action_spec = self.merge_task_actions(task)?;

        let mut node =
            ExecutionNode::new(task.id.clone(), task.name.clone(), action_spec);

        // 设置节点类型
        node.node_type = self.determine_node_type(task);

        // 提取依赖关系
        node.dependencies = self.extract_dependencies(task)?;

        // 提取执行条件
        node.condition = self.extract_condition(task)?;

        // 设置优先级
        node.priority = self.determine_priority(task)?;

        // 设置重试配置
        if let Some(retry_config) = self.extract_retry_config(task)? {
            node.retry_config = Some(retry_config);
        }

        // 设置超时配置
        if let Some(timeout_config) = self.extract_timeout_config(task)? {
            node.timeout_config = Some(timeout_config);
        }

        Ok(node)
    }

    /// 合并任务中的所有动作
    fn merge_task_actions(&self, task: &TaskDefinition) -> Result<ActionSpec> {
        if task.actions.is_empty() {
            return Err(anyhow::anyhow!("任务 {} 没有动作", task.id));
        }

        // 如果只有一个动作，直接使用
        if task.actions.len() == 1 {
            return self.convert_action_to_spec(&task.actions[0].action);
        }

        // 多个动作时，创建一个复合动作
        let mut parameters = HashMap::new();
        let mut outputs = HashMap::new();

        for (index, action_wrapper) in task.actions.iter().enumerate() {
            let action = &action_wrapper.action;

            // 为每个动作添加前缀
            let prefix = format!("action_{index}");

            for (key, value) in &action.parameters {
                parameters
                    .insert(format!("{prefix}_{key}"), value.value.clone());
            }

            for (key, value) in &action.outputs {
                outputs.insert(format!("{prefix}_{key}"), value.clone());
            }
        }

        Ok(ActionSpec {
            action_type: "composite".to_string(),
            parameters,
            outputs,
        })
    }

    /// 将动作定义转换为动作规格
    fn convert_action_to_spec(
        &self,
        action: &ActionDefinition,
    ) -> Result<ActionSpec> {
        let mut parameters = HashMap::new();
        for (key, param) in &action.parameters {
            parameters.insert(key.clone(), param.value.clone());
        }

        Ok(ActionSpec {
            action_type: format!("{:?}", action.action_type).to_lowercase(),
            parameters,
            outputs: action.outputs.clone(),
        })
    }

    /// 确定节点类型
    fn determine_node_type(&self, task: &TaskDefinition) -> NodeType {
        // 检查是否有条件逻辑
        for action_wrapper in &task.actions {
            let action = &action_wrapper.action;
            if action.flow.next_if.is_some() {
                return NodeType::Condition;
            }
            if action.flow.while_util.is_some() {
                return NodeType::Loop;
            }
        }

        // 检查是否有分支逻辑
        for action_wrapper in &task.actions {
            let action = &action_wrapper.action;
            if action.flow.next.is_some()
                && action.flow.next.as_ref().unwrap() != "null"
            {
                return NodeType::Branch;
            }
        }

        // 默认为动作节点
        NodeType::Action
    }

    /// 提取任务依赖关系
    /// 
    /// 新语义：当 A.next = B 时，意味着 B 依赖于 A（即 A 执行完后执行 B）
    /// 因此需要查找所有指向当前任务的其他任务，将它们作为当前任务的依赖
    fn extract_dependencies(
        &self,
        task: &TaskDefinition,
    ) -> Result<Vec<String>> {
        let mut deps = Vec::new();

        // 遍历所有任务，查找哪些任务的 flow.next 指向当前任务
        for task_wrapper in &self.config.workflow.tasks {
            let other_task = &task_wrapper.task;
            
            // 跳过当前任务自身
            if other_task.id == task.id {
                continue;
            }

            // 检查该任务的所有动作是否有 next 指向当前任务
            for action_wrapper in &other_task.actions {
                let action = &action_wrapper.action;

                if let Some(next) = &action.flow.next {
                    if next == &task.id {
                        // 找到指向当前任务的任务，将其作为依赖
                        deps.push(other_task.id.clone());
                        break; // 每个任务最多添加一次依赖
                    }
                }
            }
        }

        // 移除重复的依赖
        deps.sort();
        deps.dedup();

        Ok(deps)
    }

    /// 提取执行条件
    fn extract_condition(
        &self,
        task: &TaskDefinition,
    ) -> Result<Option<String>> {
        // 从第一个动作的条件中提取
        if let Some(action_wrapper) = task.actions.first() {
            Ok(action_wrapper.action.flow.next_if.clone())
        } else {
            Ok(None)
        }
    }

    /// 确定优先级
    fn determine_priority(&self, task: &TaskDefinition) -> Result<u32> {
        // 根据任务名称或描述确定优先级
        let name_lower = task.name.to_lowercase();
        let desc_lower = task.description.to_lowercase();

        if name_lower.contains("critical") || desc_lower.contains("critical") {
            Ok(1) // 最高优先级
        } else if name_lower.contains("urgent") || desc_lower.contains("urgent")
        {
            Ok(2)
        } else if name_lower.contains("high") || desc_lower.contains("high") {
            Ok(10)
        } else if name_lower.contains("low") || desc_lower.contains("low") {
            Ok(200)
        } else {
            Ok(100) // 默认优先级
        }
    }

    /// 提取重试配置
    fn extract_retry_config(
        &self,
        task: &TaskDefinition,
    ) -> Result<Option<RetryConfig>> {
        // 从第一个动作的重试配置中提取
        if let Some(action_wrapper) = task.actions.first() {
            if let Some(retry) = &action_wrapper.action.flow.retry {
                let strategy = if retry.delay > 0 {
                    RetryStrategy::Fixed
                } else {
                    RetryStrategy::Exponential { multiplier: 2.0 }
                };

                return Ok(Some(RetryConfig {
                    max_retries: retry.max_retries,
                    delay: retry.delay,
                    strategy,
                }));
            }
        }

        Ok(None)
    }

    /// 提取超时配置
    fn extract_timeout_config(
        &self,
        task: &TaskDefinition,
    ) -> Result<Option<TimeoutConfig>> {
        // 从第一个动作的超时配置中提取
        if let Some(action_wrapper) = task.actions.first() {
            if let Some(timeout) = &action_wrapper.action.flow.timeout {
                return Ok(Some(TimeoutConfig {
                    duration: timeout.duration,
                    on_timeout: action_wrapper.action.flow.on_timeout.clone(),
                }));
            }
        }

        Ok(None)
    }

    /// 获取环境变量
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        self.config.workflow.env.clone()
    }

    /// 获取流程变量
    pub fn get_flow_vars(&self) -> HashMap<String, serde_yaml::Value> {
        self.config.workflow.vars.clone()
    }

    /// 获取工作流名称
    pub fn get_workflow_name(&self) -> String {
        self.config
            .workflow
            .vars
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Workflow")
            .to_string()
    }

    /// 获取工作流版本
    pub fn get_workflow_version(&self) -> String {
        self.config.workflow.version.clone()
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        if self.config.workflow.tasks.is_empty() {
            return Err(anyhow::anyhow!("工作流没有任务"));
        }

        for task_wrapper in &self.config.workflow.tasks {
            let task = &task_wrapper.task;

            if task.id.is_empty() {
                return Err(anyhow::anyhow!("任务ID不能为空"));
            }

            if task.name.is_empty() {
                return Err(anyhow::anyhow!("任务名称不能为空"));
            }

            if task.actions.is_empty() {
                return Err(anyhow::anyhow!("任务 {} 没有动作", task.id));
            }

            // 验证每个动作
            for action_wrapper in &task.actions {
                let action = &action_wrapper.action;

                if action.id.is_empty() {
                    return Err(anyhow::anyhow!("动作ID不能为空"));
                }

                if action.name.is_empty() {
                    return Err(anyhow::anyhow!("动作名称不能为空"));
                }
            }
        }

        Ok(())
    }
}

impl ConfigParser<WorkflowConfig> for YamlConfigParser {
    type Output = Vec<ExecutionNode>;
    type Error = anyhow::Error;

    fn parse(
        &self,
        config: WorkflowConfig,
    ) -> Result<Self::Output, Self::Error> {
        let parser = YamlConfigParser::new(config);
        parser.parse()
    }
}

/// 配置解析结果
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// 执行节点列表
    pub nodes: Vec<ExecutionNode>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 流程变量
    pub flow_vars: HashMap<String, serde_yaml::Value>,
    /// 工作流名称
    pub workflow_name: String,
    /// 工作流版本
    pub workflow_version: String,
}

impl YamlConfigParser {
    /// 解析配置并返回完整结果
    pub fn parse_full(&self) -> Result<ParseResult> {
        let nodes = self.parse()?;

        Ok(ParseResult {
            nodes,
            env_vars: self.get_env_vars(),
            flow_vars: self.get_flow_vars(),
            workflow_name: self.get_workflow_name(),
            workflow_version: self.get_workflow_version(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::WorkflowLoader;

    #[test]
    fn test_yaml_parser_creation() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "test"
  vars:
    name: "Test Workflow"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions:
          - action:
              id: "action1"
              name: "Test Action"
              description: "A test action"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let parser = YamlConfigParser::new(config);

        assert!(parser.validate().is_ok());
    }

    #[test]
    fn test_parse_nodes() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "test"
  vars:
    name: "Test Workflow"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions:
          - action:
              id: "action1"
              name: "Test Action"
              description: "A test action"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let parser = YamlConfigParser::new(config);
        let nodes = parser.parse().unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "task1");
        assert_eq!(nodes[0].name, "Test Task");
        assert_eq!(nodes[0].action_spec.action_type, "builtin");
    }

    #[test]
    fn test_parse_full_result() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "test"
  vars:
    name: "Test Workflow"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions:
          - action:
              id: "action1"
              name: "Test Action"
              description: "A test action"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let parser = YamlConfigParser::new(config);
        let result = parser.parse_full().unwrap();

        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.workflow_name, "Test Workflow");
        assert_eq!(result.workflow_version, "1.0");
        assert!(result.env_vars.contains_key("TEST_ENV"));
        assert!(result.flow_vars.contains_key("name"));
    }

    #[test]
    fn test_dependency_chain_semantics() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env: {}
  vars:
    name: "Dependency Chain Test"
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
        let parser = YamlConfigParser::new(config);
        let nodes = parser.parse().unwrap();

        assert_eq!(nodes.len(), 3);

        // Find each node by id
        let setup_node = nodes.iter().find(|n| n.id == "setup_task").unwrap();
        let notification_node = nodes.iter().find(|n| n.id == "notification_task").unwrap();
        let process_node = nodes.iter().find(|n| n.id == "process_task").unwrap();

        // Test correct dependency semantics:
        // setup_task.next -> notification_task.next -> process_task.next -> null
        // Should result in:
        // - setup_task has deps: []
        // - notification_task has deps: [setup_task]  
        // - process_task has deps: [notification_task]

        assert_eq!(setup_node.dependencies, Vec::<String>::new(), 
                  "setup_task should have no dependencies");
        assert_eq!(notification_node.dependencies, vec!["setup_task"], 
                  "notification_task should depend on setup_task");
        assert_eq!(process_node.dependencies, vec!["notification_task"], 
                  "process_task should depend on notification_task");
    }

    #[test]
    fn test_independent_tasks_no_dependencies() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  env: {}
  vars:
    name: "Independent Tasks Test"
  tasks:
    - task:
        id: "task_a"
        name: "Task A"
        description: "Independent task A"
        actions:
          - action:
              id: "action_a"
              name: "Action A"
              description: "Action A"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
    - task:
        id: "task_b"
        name: "Task B"
        description: "Independent task B"
        actions:
          - action:
              id: "action_b"
              name: "Action B"
              description: "Action B"
              type: "builtin"
              flow:
                next: null
              outputs: {}
              parameters: {}
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        let parser = YamlConfigParser::new(config);
        let nodes = parser.parse().unwrap();

        assert_eq!(nodes.len(), 2);

        // Both tasks should have no dependencies since they have next: null
        for node in &nodes {
            assert_eq!(node.dependencies, Vec::<String>::new(), 
                      "Tasks with next: null should have no dependencies");
        }
    }
}
