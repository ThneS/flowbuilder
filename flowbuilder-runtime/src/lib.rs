//! # FlowBuilder Runtime
//!
//! Advanced runtime features for FlowBuilder including
//! task scheduling and flow orchestration

mod orchestrator_simple;
mod scheduler_simple;

use orchestrator_simple as orchestrator;
use scheduler_simple as scheduler;

#[cfg(test)]
mod tests;

// Re-export orchestrator types
pub use orchestrator::{
    BranchCondition, Checkpoint, ErrorRecoveryStrategy, ExecutionStats, FlowNode, FlowOrchestrator,
    FlowState, OrchestratorConfig, RetryConfig,
};

// Re-export scheduler types
pub use scheduler::{
    Priority, ScheduledTask, SchedulerConfig, SchedulerStats, SchedulingStrategy, TaskScheduler,
    TaskStatus,
};

use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_core::FlowBuilder;
use std::sync::Arc;

/// Runtime extensions for FlowBuilder
pub trait FlowBuilderExt {
    /// Schedule execution with a task scheduler
    fn schedule_with(self, scheduler: &mut TaskScheduler) -> Result<String>;

    /// Execute with orchestration capabilities
    fn orchestrate_with(self, orchestrator: &mut FlowOrchestrator) -> Result<()>;
}

impl FlowBuilderExt for FlowBuilder {
    fn schedule_with(self, _scheduler: &mut TaskScheduler) -> Result<String> {
        let _task = ScheduledTask::new(
            "flow_task".to_string(),
            Arc::new(move || {
                // This is a simplified execution - in a real implementation
                // we would need to properly handle the FlowBuilder execution
                Ok(())
            }),
            Priority::Normal,
        );

        // We need to implement add_task method
        Ok("task_id".to_string())
    }

    fn orchestrate_with(self, orchestrator: &mut FlowOrchestrator) -> Result<()> {
        // Create a simple flow node from the FlowBuilder
        let node = FlowNode::new("flow_node".to_string()).name("FlowBuilder Execution".to_string());

        orchestrator.add_node(node);
        Ok(())
    }
}

/// Runtime helper functions
pub mod runtime {
    use super::*;

    /// Create a new task scheduler with default configuration
    pub fn create_scheduler() -> TaskScheduler {
        TaskScheduler::new(SchedulerConfig::default())
    }

    /// Create a new task scheduler with custom configuration
    pub fn create_scheduler_with_config(config: SchedulerConfig) -> TaskScheduler {
        TaskScheduler::new(config)
    }

    /// Create a new flow orchestrator with default configuration
    pub fn create_orchestrator() -> FlowOrchestrator {
        FlowOrchestrator::new()
    }

    /// Create a new flow orchestrator with custom configuration
    pub fn create_orchestrator_with_config(config: OrchestratorConfig) -> FlowOrchestrator {
        FlowOrchestrator::with_config(config)
    }
}

/// Integration utilities
pub mod integration {
    use super::*;

    /// Execute a FlowBuilder with both scheduling and orchestration
    pub async fn execute_with_runtime(
        flow_builder: FlowBuilder,
        scheduler_config: Option<SchedulerConfig>,
        orchestrator_config: Option<OrchestratorConfig>,
    ) -> Result<FlowContext> {
        let mut task_scheduler = match scheduler_config {
            Some(config) => TaskScheduler::new(config),
            None => TaskScheduler::new(SchedulerConfig::default()),
        };

        let _orchestrator = match orchestrator_config {
            Some(config) => FlowOrchestrator::with_config(config),
            None => FlowOrchestrator::new(),
        };

        // Schedule the flow
        let _task_id = flow_builder.schedule_with(&mut task_scheduler)?;

        // Start the scheduler
        task_scheduler.start().await?;

        // For now, return a default context
        // In a full implementation, we would integrate the scheduler and orchestrator results
        Ok(FlowContext::default())
    }
}
