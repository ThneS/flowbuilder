use crate::Step;
use flowbuilder_context::SharedContext;
use anyhow::Result;

/// Executes flow steps
pub struct FlowExecutor;

impl FlowExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a list of steps sequentially
    pub async fn execute_steps(&self, steps: Vec<Step>, context: SharedContext) -> Result<()> {
        for step in steps {
            step(context.clone()).await?;
        }
        Ok(())
    }
}

impl Default for FlowExecutor {
    fn default() -> Self {
        Self::new()
    }
}
