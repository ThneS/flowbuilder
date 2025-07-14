//! # FlowBuilder Runtime
//!
//! 高级运行时功能，包括任务调度、流程编排和增强执行器

mod enhanced_orchestrator;
mod enhanced_executor;

// 重新导出增强组件
pub use enhanced_orchestrator::{
    EnhancedFlowOrchestrator, OrchestratorConfig as EnhancedOrchestratorConfig,
    ExecutionComplexity,
};

pub use enhanced_executor::{
    EnhancedTaskExecutor, ExecutorConfig, ExecutionStats,
    ExecutionResult, PhaseResult, NodeResult,
};

/// 预导入模块
pub mod prelude {
    // 增强组件
    pub use crate::{
        EnhancedFlowOrchestrator, EnhancedTaskExecutor, ExecutionResult,
        PhaseResult, NodeResult, ExecutionComplexity,
    };

    // 核心接口
    pub use flowbuilder_core::{
        ExecutionPlan, ExecutionPhase, ExecutionNode, Executor, ExecutorStatus,
        FlowPlanner,
    };
}
