//! # FlowBuilder Context
//!
//! Context management and shared state for FlowBuilder

use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FlowContext {
    pub trace_id: String,
    pub ok: bool,
    pub errors: Vec<String>,
    pub step_logs: Vec<StepLog>,
    pub variables: std::collections::HashMap<String, String>,
    pub snapshots: std::collections::HashMap<String, ContextSnapshot>,
}

#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub snapshot_id: String,
    pub timestamp: std::time::Instant,
    pub variables: std::collections::HashMap<String, String>,
    pub ok: bool,
    pub errors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct StepLog {
    pub step_name: String,
    pub start_time: std::time::Instant,
    pub end_time: Option<std::time::Instant>,
    pub status: StepStatus,
    pub error_message: Option<String>,
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub enum StepStatus {
    Running,
    Success,
    Failed,
    Skipped,
    Timeout,
}

impl Default for FlowContext {
    fn default() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            ok: true,
            errors: Vec::new(),
            step_logs: Vec::new(),
            variables: std::collections::HashMap::new(),
            snapshots: std::collections::HashMap::new(),
        }
    }
}

impl FlowContext {
    pub fn new_with_trace_id(trace_id: String) -> Self {
        Self {
            trace_id,
            ok: true,
            errors: Vec::new(),
            step_logs: Vec::new(),
            variables: std::collections::HashMap::new(),
            snapshots: std::collections::HashMap::new(),
        }
    }

    /// 创建快照
    pub fn create_snapshot(
        &mut self,
        snapshot_id: String,
        description: String,
    ) -> Result<(), String> {
        if self.snapshots.contains_key(&snapshot_id) {
            return Err(format!(
                "Snapshot with id '{snapshot_id}' already exists"
            ));
        }

        let snapshot = ContextSnapshot {
            snapshot_id: snapshot_id.clone(),
            timestamp: std::time::Instant::now(),
            variables: self.variables.clone(),
            ok: self.ok,
            errors: self.errors.clone(),
            description,
        };

        self.snapshots.insert(snapshot_id.clone(), snapshot);

        tracing::info!(trace_id = %self.trace_id, snapshot = %snapshot_id, "Created snapshot");

        Ok(())
    }

    /// 回滚到快照
    pub fn rollback_to_snapshot(
        &mut self,
        snapshot_id: &str,
    ) -> Result<(), String> {
        let snapshot = self
            .snapshots
            .get(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?
            .clone();

        // 保留 trace_id 和快照信息，回滚其他状态
        let old_variables_count = self.variables.len();
        let old_errors_count = self.errors.len();

        self.variables = snapshot.variables;
        self.ok = snapshot.ok;
        self.errors = snapshot.errors;

        tracing::info!(
            trace_id = %self.trace_id,
            snapshot = %snapshot_id,
            description = %snapshot.description,
            old_variables = old_variables_count,
            new_variables = self.variables.len(),
            old_errors = old_errors_count,
            new_errors = self.errors.len(),
            "Rolled back to snapshot"
        );

        Ok(())
    }

    /// 删除快照
    pub fn remove_snapshot(&mut self, snapshot_id: &str) -> Result<(), String> {
        self.snapshots
            .remove(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?;

        tracing::info!(trace_id = %self.trace_id, snapshot = %snapshot_id, "Removed snapshot");

        Ok(())
    }

    /// 列出所有快照
    pub fn list_snapshots(&self) -> Vec<&ContextSnapshot> {
        self.snapshots.values().collect()
    }

    pub fn start_step(&mut self, step_name: String) {
        let step_log = StepLog {
            step_name: step_name.clone(),
            start_time: std::time::Instant::now(),
            end_time: None,
            status: StepStatus::Running,
            error_message: None,
            trace_id: self.trace_id.clone(),
        };
        self.step_logs.push(step_log);

        tracing::info!(trace_id = %self.trace_id, step = %step_name, "step starting");
    }

    pub fn end_step_success(&mut self, step_name: &str) {
        if let Some(log) = self
            .step_logs
            .iter_mut()
            .rev()
            .find(|log| log.step_name == step_name)
        {
            log.end_time = Some(std::time::Instant::now());
            log.status = StepStatus::Success;
            let duration = log.end_time.unwrap().duration_since(log.start_time);

            tracing::info!(trace_id = %self.trace_id, step = %step_name, duration_ms = ?duration, "step success");
        }
    }

    pub fn end_step_failed(&mut self, step_name: &str, error: &str) {
        if let Some(log) = self
            .step_logs
            .iter_mut()
            .rev()
            .find(|log| log.step_name == step_name)
        {
            log.end_time = Some(std::time::Instant::now());
            log.status = StepStatus::Failed;
            log.error_message = Some(error.to_string());
            let duration = log.end_time.unwrap().duration_since(log.start_time);

            tracing::error!(trace_id = %self.trace_id, step = %step_name, duration_ms = ?duration, error = %error, "step failed");
        }
        self.errors
            .push(format!("[{}] {}: {}", self.trace_id, step_name, error));
    }

    pub fn end_step_skipped(&mut self, step_name: &str, reason: &str) {
        if let Some(log) = self
            .step_logs
            .iter_mut()
            .rev()
            .find(|log| log.step_name == step_name)
        {
            log.end_time = Some(std::time::Instant::now());
            log.status = StepStatus::Skipped;
            let duration = log.end_time.unwrap().duration_since(log.start_time);

            tracing::warn!(trace_id = %self.trace_id, step = %step_name, duration_ms = ?duration, reason = %reason, "step skipped");
        }
    }

    pub fn end_step_timeout(&mut self, step_name: &str) {
        if let Some(log) = self
            .step_logs
            .iter_mut()
            .rev()
            .find(|log| log.step_name == step_name)
        {
            log.end_time = Some(std::time::Instant::now());
            log.status = StepStatus::Timeout;
            let duration = log.end_time.unwrap().duration_since(log.start_time);

            tracing::error!(trace_id = %self.trace_id, step = %step_name, duration_ms = ?duration, "step timeout");
        }
        self.errors
            .push(format!("[{}] {}: timeout", self.trace_id, step_name));
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        tracing::debug!(trace_id = %self.trace_id, key = %key, value = %value, "set variable");

        self.variables.insert(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn print_summary(&self) {
        let summary =
            format!("\n=== Flow Summary [trace_id: {}] ===", self.trace_id);

        tracing::info!(summary = %summary);
        tracing::info!(total_steps = self.step_logs.len(), "steps summary");

        let success_count = self
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Success))
            .count();
        let failed_count = self
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Failed))
            .count();
        let skipped_count = self
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Skipped))
            .count();
        let timeout_count = self
            .step_logs
            .iter()
            .filter(|log| matches!(log.status, StepStatus::Timeout))
            .count();

        tracing::info!(
            success = success_count,
            failed = failed_count,
            skipped = skipped_count,
            timeout = timeout_count
        );

        if !self.errors.is_empty() {
            tracing::info!(errors = self.errors.len(), "errors summary");
            for error in &self.errors {
                tracing::info!(error = %error);
            }
        }

        if !self.variables.is_empty() {
            tracing::info!(vars = self.variables.len(), "variables summary");
            for (key, value) in &self.variables {
                tracing::debug!(key = %key, value = %value);
            }
        }
        tracing::info!("==============================");
    }
}

pub type SharedContext = Arc<Mutex<FlowContext>>;
