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

    // 创建快照
    pub fn create_snapshot(
        &mut self,
        snapshot_id: String,
        description: String,
    ) -> Result<(), String> {
        if self.snapshots.contains_key(&snapshot_id) {
            return Err(format!("Snapshot with id '{}' already exists", snapshot_id));
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
        println!(
            "[trace_id:{}] Created snapshot: {}",
            self.trace_id, snapshot_id
        );
        Ok(())
    }

    // 回滚到快照
    pub fn rollback_to_snapshot(&mut self, snapshot_id: &str) -> Result<(), String> {
        let snapshot = self
            .snapshots
            .get(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{}' not found", snapshot_id))?
            .clone();

        // 保留 trace_id 和快照信息，回滚其他状态
        let old_variables_count = self.variables.len();
        let old_errors_count = self.errors.len();

        self.variables = snapshot.variables;
        self.ok = snapshot.ok;
        self.errors = snapshot.errors;

        println!(
            "[trace_id:{}] Rolled back to snapshot '{}' ({}). Variables: {} -> {}, Errors: {} -> {}",
            self.trace_id,
            snapshot_id,
            snapshot.description,
            old_variables_count,
            self.variables.len(),
            old_errors_count,
            self.errors.len()
        );

        Ok(())
    }

    // 删除快照
    pub fn remove_snapshot(&mut self, snapshot_id: &str) -> Result<(), String> {
        self.snapshots
            .remove(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{}' not found", snapshot_id))?;

        println!(
            "[trace_id:{}] Removed snapshot: {}",
            self.trace_id, snapshot_id
        );
        Ok(())
    }

    // 列出所有快照
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
        println!(
            "[trace_id:{}] [step:{}] starting...",
            self.trace_id, step_name
        );
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
            println!(
                "[trace_id:{}] [step:{}] completed successfully in {:?}",
                self.trace_id, step_name, duration
            );
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
            println!(
                "[trace_id:{}] [step:{}] failed after {:?}: {}",
                self.trace_id, step_name, duration, error
            );
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
            println!(
                "[trace_id:{}] [step:{}] skipped after {:?}: {}",
                self.trace_id, step_name, duration, reason
            );
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
            println!(
                "[trace_id:{}] [step:{}] timed out after {:?}",
                self.trace_id, step_name, duration
            );
        }
        self.errors
            .push(format!("[{}] {}: timeout", self.trace_id, step_name));
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        println!(
            "[trace_id:{}] setting variable {} = {}",
            self.trace_id, key, value
        );
        self.variables.insert(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn print_summary(&self) {
        println!("\n=== Flow Summary [trace_id: {}] ===", self.trace_id);
        println!("Total steps: {}", self.step_logs.len());

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

        println!(
            "Success: {}, Failed: {}, Skipped: {}, Timeout: {}",
            success_count, failed_count, skipped_count, timeout_count
        );

        if !self.errors.is_empty() {
            println!("Errors: {}", self.errors.len());
            for error in &self.errors {
                println!("  - {}", error);
            }
        }

        if !self.variables.is_empty() {
            println!("Variables:");
            for (key, value) in &self.variables {
                println!("  {} = {}", key, value);
            }
        }
        println!("==============================\n");
    }
}

pub type SharedContext = Arc<Mutex<FlowContext>>;
