use crate::config::WorkflowConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// 工作流加载器，支持从文件或字符串加载配置
pub struct WorkflowLoader;

impl WorkflowLoader {
    /// 从 YAML 文件加载工作流配置
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<WorkflowConfig> {
        let content = fs::read_to_string(&path).with_context(|| {
            format!("Failed to read YAML file: {:?}", path.as_ref())
        })?;
        Self::from_yaml_str(&content)
    }

    /// 从 YAML 字符串加载工作流配置
    pub fn from_yaml_str(content: &str) -> Result<WorkflowConfig> {
        serde_yaml::from_str(content)
            .with_context(|| "Failed to parse YAML content")
    }

    /// 从 JSON 文件加载工作流配置
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<WorkflowConfig> {
        let content = fs::read_to_string(&path).with_context(|| {
            format!("Failed to read JSON file: {:?}", path.as_ref())
        })?;
        Self::from_json_str(&content)
    }

    /// 从 JSON 字符串加载工作流配置
    pub fn from_json_str(content: &str) -> Result<WorkflowConfig> {
        serde_json::from_str(content)
            .with_context(|| "Failed to parse JSON content")
    }

    /// 保存工作流配置到 YAML 文件
    pub fn save_to_yaml<P: AsRef<Path>>(
        config: &WorkflowConfig,
        path: P,
    ) -> Result<()> {
        let yaml_content = serde_yaml::to_string(config)
            .with_context(|| "Failed to serialize config to YAML")?;

        fs::write(&path, yaml_content).with_context(|| {
            format!("Failed to write YAML file: {:?}", path.as_ref())
        })?;

        Ok(())
    }

    /// 保存工作流配置到 JSON 文件
    pub fn save_to_json<P: AsRef<Path>>(
        config: &WorkflowConfig,
        path: P,
    ) -> Result<()> {
        let json_content = serde_json::to_string_pretty(config)
            .with_context(|| "Failed to serialize config to JSON")?;

        fs::write(&path, json_content).with_context(|| {
            format!("Failed to write JSON file: {:?}", path.as_ref())
        })?;

        Ok(())
    }

    /// 验证工作流配置的基本有效性
    pub fn validate(config: &WorkflowConfig) -> Result<()> {
        let workflow = &config.workflow;

        // 检查版本格式
        if workflow.version.is_empty() {
            return Err(anyhow::anyhow!("Workflow version cannot be empty"));
        }

        // 检查任务定义
        if workflow.tasks.is_empty() {
            return Err(anyhow::anyhow!(
                "Workflow must contain at least one task"
            ));
        }

        // 检查任务 ID 的唯一性
        let mut task_ids = std::collections::HashSet::new();
        for task in &workflow.tasks {
            if !task_ids.insert(&task.task.id) {
                return Err(anyhow::anyhow!(
                    "Duplicate task ID: {}",
                    task.task.id
                ));
            }
        }

        // 检查动作 ID 的唯一性
        let mut action_ids = std::collections::HashSet::new();
        for task in &workflow.tasks {
            for action in &task.task.actions {
                let full_action_id =
                    format!("{}.{}", task.task.id, action.action.id);
                if !action_ids.insert(full_action_id.clone()) {
                    return Err(anyhow::anyhow!(
                        "Duplicate action ID: {}",
                        full_action_id
                    ));
                }
            }
        }

        Ok(())
    }

    /// 从配置创建带有 runtime 功能的执行器
    pub fn create_runtime_executor(
        config: WorkflowConfig,
    ) -> Result<crate::executor::DynamicFlowExecutor> {
        use crate::executor::DynamicFlowExecutor;
        DynamicFlowExecutor::new(config)
    }

    /// 快速执行工作流文件（使用 runtime 功能）
    pub async fn execute_workflow_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<()> {
        let config = Self::from_yaml_file(path)?;
        Self::validate(&config)?;

        let mut executor = Self::create_runtime_executor(config)?;
        let context = std::sync::Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        // 使用统一的执行接口
        executor.execute(context).await?;

        Ok(())
    }

    /// 批量执行多个工作流文件（简化版本）
    pub async fn execute_workflow_batch<P: AsRef<Path>>(
        paths: Vec<P>,
        max_concurrent: usize,
    ) -> Result<Vec<Result<()>>> {
        let semaphore =
            std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut results = Vec::new();

        for path in paths {
            let _permit = semaphore.acquire().await.map_err(|e| {
                anyhow::anyhow!("Failed to acquire semaphore: {}", e)
            })?;
            let result = Self::execute_workflow_file(path).await;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_yaml_str() {
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
        actions: []
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        assert_eq!(config.workflow.version, "1.0");
        assert_eq!(
            config.workflow.env.get("FLOWBUILDER_ENV"),
            Some(&"test".to_string())
        );
        assert_eq!(config.workflow.tasks.len(), 1);
        assert_eq!(config.workflow.tasks[0].task.id, "task1");
    }

    #[test]
    fn test_validate_config() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  tasks:
    - task:
        id: "task1"
        name: "Test Task"
        description: "A test task"
        actions: []
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        assert!(WorkflowLoader::validate(&config).is_ok());
    }

    #[test]
    fn test_validate_duplicate_task_ids() {
        let yaml_content = r#"
workflow:
  version: "1.0"
  tasks:
    - task:
        id: "task1"
        name: "Test Task 1"
        description: "A test task"
        actions: []
    - task:
        id: "task1"
        name: "Test Task 2"
        description: "Another test task"
        actions: []
"#;

        let config = WorkflowLoader::from_yaml_str(yaml_content).unwrap();
        assert!(WorkflowLoader::validate(&config).is_err());
    }
}
