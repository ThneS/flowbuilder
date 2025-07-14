//! # FlowBuilder Core
//!
//! 核心流程构建功能，包括执行计划、节点定义和执行器接口

#![cfg_attr(docsrs, feature(doc_cfg))]

mod execution_plan;
mod executor;
mod flow;
mod flow_builder;

#[cfg(test)]
mod tests;

// 原有的公共接口
pub use executor::FlowExecutor;
pub use flow::Flow;
pub use flow_builder::{FlowBuilder, Step, StepFuture};

// 新架构的公共接口
pub use execution_plan::{
    ActionSpec, ConfigParser, ExecutionNode, ExecutionPhase, ExecutionPlan,
    Executor, ExecutorStatus, ExpressionEvaluator, FlowPlanner, NodeType,
    PhaseExecutionMode, PlanMetadata, RetryConfig, RetryStrategy,
    TimeoutConfig,
};

/// 预导入模块
pub mod prelude {
    // 原有接口
    pub use crate::{Flow, FlowBuilder, FlowExecutor, Step, StepFuture};
    pub use flowbuilder_context::{FlowContext, SharedContext};

    // 新架构接口
    pub use crate::{
        ConfigParser, ExecutionNode, ExecutionPhase, ExecutionPlan, Executor,
        ExecutorStatus, ExpressionEvaluator, FlowPlanner, PhaseExecutionMode,
    };
}
