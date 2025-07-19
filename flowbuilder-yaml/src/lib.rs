//! # FlowBuilder YAML
//!
//! 动态流程构建，从YAML/JSON配置文件构建工作流

mod config;
mod config_parser;
mod executor;
mod expression;
mod loader;
mod parser;

// 重新导出主要类型
pub use config::*;
pub use config_parser::*;
pub use executor::*;
pub use expression::*;
pub use loader::*;
pub use parser::*;

/// 预导入模块
pub mod prelude {
    pub use crate::{
        DynamicFlowExecutor, ExpressionEvaluator, WorkflowConfig, WorkflowInfo,
        WorkflowLoader, YamlConfigParser,
    };
    pub use flowbuilder_core::prelude::*;
    pub use flowbuilder_runtime::{
        EnhancedFlowOrchestrator, EnhancedTaskExecutor, ExecutionResult,
    };
}
