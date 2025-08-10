//! # FlowBuilder Runtime
//!
//! 高级运行时功能，包括任务调度、流程编排和增强执行器

mod enhanced_executor;
mod enhanced_orchestrator;

// 重新导出增强组件
pub use enhanced_orchestrator::{
    EnhancedFlowOrchestrator, ExecutionComplexity,
    OrchestratorConfig as EnhancedOrchestratorConfig,
};

pub use enhanced_executor::{
    EnhancedTaskExecutor, ExecutionResult, ExecutorConfig, NodeResult,
    PhaseResult,
};

#[cfg(feature = "perf-metrics")]
pub use enhanced_executor::ExecutionStats;

/// 预导入模块
pub mod prelude {
    // 增强组件
    pub use crate::{
        EnhancedFlowOrchestrator, EnhancedTaskExecutor, ExecutionComplexity,
        ExecutionResult, NodeResult, PhaseResult,
    };

    #[cfg(feature = "perf-metrics")]
    pub use crate::ExecutionStats;

    // 核心接口
    pub use flowbuilder_core::{
        ExecutionNode, ExecutionPhase, ExecutionPlan, Executor, ExecutorStatus,
        FlowPlanner,
    };
}
