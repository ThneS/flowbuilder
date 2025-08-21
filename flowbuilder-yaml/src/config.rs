use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 完整的工作流配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub workflow: Workflow,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub version: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub vars: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub template: Option<Template>,
    pub tasks: Vec<Task>,
}

/// 模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task: TaskDefinition,
}

/// 任务定义详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub actions: Vec<Action>,
}

/// 动作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action: ActionDefinition,
}

/// 动作定义详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub flow: FlowControl,
    #[serde(default)]
    pub outputs: HashMap<String, serde_yaml::Value>,
    #[serde(rename = "type")]
    pub action_type: ActionType,
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,
}

/// 流程控制
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowControl {
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub next_if: Option<String>,
    #[serde(default)]
    pub while_util: Option<WhileUtil>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
    #[serde(default)]
    pub timeout: Option<TimeoutConfig>,
    #[serde(default)]
    pub on_error: Option<String>,
    #[serde(default)]
    pub on_timeout: Option<String>,
}

/// 循环控制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileUtil {
    pub condition: String,
    pub max_iterations: u32,
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub delay: u64, // milliseconds
}

/// 超时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub duration: u64, // milliseconds
}

/// 动作类型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Cmd,
    Http,
    #[default]
    Builtin,
    Wasm,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Parameter {
    Full {
        value: serde_yaml::Value,
        #[serde(default)]
        required: bool,
    },
    Bare(serde_yaml::Value),
}

impl Parameter {
    pub fn as_value(&self) -> &serde_yaml::Value {
        match self {
            Parameter::Full { value, .. } => value,
            Parameter::Bare(v) => v,
        }
    }

    pub fn to_value(&self) -> serde_yaml::Value {
        self.as_value().clone()
    }

    pub fn is_required(&self) -> bool {
        match self {
            Parameter::Full { required, .. } => *required,
            Parameter::Bare(_) => false,
        }
    }
}
