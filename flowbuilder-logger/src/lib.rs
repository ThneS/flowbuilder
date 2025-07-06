//! # FlowBuilder Logger
//!
//! Logging and tracing support for FlowBuilder flows

use flowbuilder_context::{FlowContext, StepStatus};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Logger for FlowBuilder flows
pub struct Logger {
    pub trace_id: String,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
        }
    }

    /// Create a logger with a specific trace ID
    pub fn with_trace_id(trace_id: String) -> Self {
        Self { trace_id }
    }

    /// Initialize tracing subscriber
    pub fn init_tracing() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        info!(trace_id = %self.trace_id, "{}", message);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        warn!(trace_id = %self.trace_id, "{}", message);
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        error!(trace_id = %self.trace_id, "{}", message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        debug!(trace_id = %self.trace_id, "{}", message);
    }

    /// Log flow context summary
    pub fn log_flow_summary(&self, context: &FlowContext) {
        let total_steps = context.step_logs.len();
        let success_count = context
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Success))
            .count();
        let failed_count = context
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Failed))
            .count();
        let skipped_count = context
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Skipped))
            .count();
        let timeout_count = context
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Timeout))
            .count();

        info!(
            trace_id = %context.trace_id,
            total_steps = total_steps,
            success = success_count,
            failed = failed_count,
            skipped = skipped_count,
            timeout = timeout_count,
            variables = context.variables.len(),
            errors = context.errors.len(),
            "Flow execution summary"
        );

        if !context.errors.is_empty() {
            for error in &context.errors {
                error!(trace_id = %context.trace_id, "Flow error: {}", error);
            }
        }
    }

    /// Log step execution details
    pub fn log_step_details(&self, context: &FlowContext) {
        for step_log in &context.step_logs {
            let duration = step_log
                .end_time
                .map(|end| end.duration_since(step_log.start_time))
                .unwrap_or_default();

            match step_log.status {
                StepStatus::Success => {
                    info!(
                        trace_id = %step_log.trace_id,
                        step_name = %step_log.step_name,
                        duration_ms = duration.as_millis(),
                        "Step completed successfully"
                    );
                }
                StepStatus::Failed => {
                    error!(
                        trace_id = %step_log.trace_id,
                        step_name = %step_log.step_name,
                        duration_ms = duration.as_millis(),
                        error = %step_log.error_message.as_deref().unwrap_or("Unknown error"),
                        "Step failed"
                    );
                }
                StepStatus::Skipped => {
                    warn!(
                        trace_id = %step_log.trace_id,
                        step_name = %step_log.step_name,
                        duration_ms = duration.as_millis(),
                        "Step skipped"
                    );
                }
                StepStatus::Timeout => {
                    error!(
                        trace_id = %step_log.trace_id,
                        step_name = %step_log.step_name,
                        duration_ms = duration.as_millis(),
                        "Step timed out"
                    );
                }
                StepStatus::Running => {
                    warn!(
                        trace_id = %step_log.trace_id,
                        step_name = %step_log.step_name,
                        "Step still running"
                    );
                }
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
