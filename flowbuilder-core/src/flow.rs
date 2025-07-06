use crate::{FlowExecutor, Step};
use anyhow::Result;
use flowbuilder_context::FlowContext;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents a flow that can be executed
pub struct Flow {
    steps: Vec<Step>,
}

impl Flow {
    pub(crate) fn new(steps: Vec<Step>) -> Self {
        Self { steps }
    }

    /// Execute the flow with a default context
    pub async fn execute(self) -> Result<FlowContext> {
        let context = FlowContext::default();
        self.execute_with_context(context).await
    }

    /// Execute the flow with a custom context
    pub async fn execute_with_context(self, context: FlowContext) -> Result<FlowContext> {
        let shared_context = Arc::new(Mutex::new(context));
        let executor = FlowExecutor::new();

        executor
            .execute_steps(self.steps, shared_context.clone())
            .await?;

        let final_context = Arc::try_unwrap(shared_context)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap shared context"))?
            .into_inner();

        Ok(final_context)
    }
}
