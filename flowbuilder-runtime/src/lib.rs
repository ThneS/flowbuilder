//! # FlowBuilder Runtime
//!
//! Advanced runtime features for FlowBuilder including parallel execution,
//! task scheduling, and flow orchestration

mod orchestrator;
mod parallel;
// mod scheduler; // TODO: 待实现高级调度功能

pub use orchestrator::FlowOrchestrator;
pub use parallel::{ParallelConfig, ParallelExecutor, ParallelResults, ParallelStep};

use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_core::FlowBuilder;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Runtime extensions for FlowBuilder
pub trait FlowBuilderExt {
    /// Execute steps in parallel with default configuration
    fn execute_parallel(self) -> ParallelFlow;

    /// Execute steps in parallel with custom configuration
    fn execute_parallel_with_config(self, config: ParallelConfig) -> ParallelFlow;
}

impl FlowBuilderExt for FlowBuilder {
    fn execute_parallel(self) -> ParallelFlow {
        ParallelFlow {
            builder: self,
            config: ParallelConfig::default(),
        }
    }

    fn execute_parallel_with_config(self, config: ParallelConfig) -> ParallelFlow {
        ParallelFlow {
            builder: self,
            config,
        }
    }
}

/// Parallel flow execution wrapper
pub struct ParallelFlow {
    builder: FlowBuilder,
    config: ParallelConfig,
}

impl ParallelFlow {
    /// Execute all steps in parallel
    pub async fn execute(self) -> Result<FlowContext> {
        let context = FlowContext::default();
        self.execute_with_context(context).await
    }

    /// Execute all steps in parallel with custom context
    pub async fn execute_with_context(self, context: FlowContext) -> Result<FlowContext> {
        let shared_context = Arc::new(Mutex::new(context));
        let executor = ParallelExecutor::with_config(self.config);

        executor
            .execute_parallel(self.builder.into_steps(), shared_context.clone())
            .await?;

        let final_context = Arc::try_unwrap(shared_context)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap shared context"))?
            .into_inner();

        Ok(final_context)
    }

    /// Execute and return detailed results
    pub async fn execute_detailed(self) -> Result<ParallelResults> {
        let context = FlowContext::default();
        self.execute_detailed_with_context(context).await
    }

    /// Execute with custom context and return detailed results
    pub async fn execute_detailed_with_context(
        self,
        context: FlowContext,
    ) -> Result<ParallelResults> {
        let shared_context = Arc::new(Mutex::new(context));
        let executor = ParallelExecutor::with_config(self.config);

        executor
            .execute_parallel_detailed(self.builder.into_steps(), shared_context)
            .await
    }

    /// Configure maximum concurrency
    pub fn max_concurrency(mut self, concurrency: usize) -> Self {
        self.config = self.config.max_concurrency(concurrency);
        self
    }

    /// Configure step timeout
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config = self.config.timeout(timeout);
        self
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self) -> Self {
        self.config = self.config.with_fail_fast();
        self
    }

    /// Configure batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config = self.config.batch_size(size);
        self
    }
}
