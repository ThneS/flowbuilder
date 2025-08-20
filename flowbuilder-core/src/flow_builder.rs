use crate::Flow;
use anyhow::Result;
use flowbuilder_context::{FlowContext, SharedContext};
use std::{future::Future, pin::Pin, time::Duration};
use tracing::{info, warn};

/// Type alias for step functions
pub type StepFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub type Step = Box<dyn FnOnce(SharedContext) -> StepFuture + Send>;

/// Builder for creating flows with a fluent API
pub struct FlowBuilder {
    steps: Vec<Step>,
}

impl Default for FlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowBuilder {
    /// Creates a new FlowBuilder
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Adds a simple step to the flow
    pub fn step<Fut, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| Box::pin(f(ctx))));
        self
    }

    /// Adds a named step to the flow with automatic logging
    pub fn named_step<Fut, F>(mut self, name: &'static str, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // Start step logging
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = f(ctx2.clone()).await;

                // End step logging
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    /// Adds a conditional step that only executes if the condition is met
    pub fn step_if<Fut, F, Cond>(mut self, cond: Cond, mut f: F) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let guard = ctx2.lock().await;
                if cond(&guard) {
                    drop(guard);
                    f(ctx2).await
                } else {
                    let trace_id = guard.trace_id.clone();
                    drop(guard);
                    warn!(trace_id = %trace_id, "[step_if] condition not met, skipping step");
                    Ok(())
                }
            })
        }));
        self
    }

    /// Adds a wait step that waits until a condition is met
    pub fn wait_until<Cond>(
        mut self,
        cond: Cond,
        interval: Duration,
        max_retry: usize,
    ) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                for attempt in 0..max_retry {
                    {
                        let guard = ctx.lock().await;
                        if cond(&guard) {
                            info!(attempt = attempt + 1, "[wait_until] condition met");
                            return Ok(());
                        }
                    }

                    if attempt < max_retry - 1 {
                        tokio::time::sleep(interval).await;
                    }
                }

                anyhow::bail!(
                    "[wait_until] condition not met after {} attempts",
                    max_retry
                )
            })
        }));
        self
    }

    /// Builds the flow
    pub fn build(self) -> Flow {
        Flow::new(self.steps)
    }

    /// Access steps for runtime extensions
    pub fn into_steps(self) -> Vec<Step> {
        self.steps
    }

    /// Builds and executes the flow immediately
    pub async fn execute(self) -> Result<FlowContext> {
        self.build().execute().await
    }

    /// Builds and executes the flow with a custom context
    pub async fn execute_with_context(
        self,
        context: FlowContext,
    ) -> Result<FlowContext> {
        self.build().execute_with_context(context).await
    }
}
