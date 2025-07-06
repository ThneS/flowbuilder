use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_core::Flow;
use std::collections::HashMap;

/// Orchestrator for managing multiple flows and their dependencies
pub struct FlowOrchestrator {
    flows: HashMap<String, Flow>,
    dependencies: HashMap<String, Vec<String>>,
}

impl FlowOrchestrator {
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Register a flow with the orchestrator
    pub fn register_flow(&mut self, id: String, flow: Flow) -> &mut Self {
        self.flows.insert(id, flow);
        self
    }

    /// Add a dependency between flows
    pub fn add_dependency(&mut self, flow_id: String, depends_on: String) -> &mut Self {
        self.dependencies
            .entry(flow_id)
            .or_insert_with(Vec::new)
            .push(depends_on);
        self
    }

    /// Execute all flows respecting dependencies
    pub async fn execute_all(&mut self) -> Result<HashMap<String, FlowContext>> {
        let mut results = HashMap::new();
        let mut executed = std::collections::HashSet::new();
        let mut executing = std::collections::HashSet::new();

        // Get all flow IDs
        let all_flows: Vec<String> = self.flows.keys().cloned().collect();

        for flow_id in all_flows {
            if !executed.contains(&flow_id) {
                let context = self
                    .execute_flow_with_deps(&flow_id, &mut executed, &mut executing, &mut results)
                    .await?;
                results.insert(flow_id.clone(), context);
            }
        }

        Ok(results)
    }

    fn execute_flow_with_deps<'a>(
        &'a mut self,
        flow_id: &'a str,
        executed: &'a mut std::collections::HashSet<String>,
        executing: &'a mut std::collections::HashSet<String>,
        results: &'a mut HashMap<String, FlowContext>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FlowContext>> + 'a>> {
        Box::pin(async move {
            // Check for circular dependencies
            if executing.contains(flow_id) {
                return Err(anyhow::anyhow!(
                    "Circular dependency detected for flow: {}",
                    flow_id
                ));
            }

            // If already executed, return the cached result
            if let Some(context) = results.get(flow_id) {
                return Ok(context.clone());
            }

            executing.insert(flow_id.to_string());

            // Execute dependencies first
            if let Some(deps) = self.dependencies.get(flow_id).cloned() {
                for dep in deps {
                    if !executed.contains(&dep) {
                        self.execute_flow_with_deps(&dep, executed, executing, results)
                            .await?;
                    }
                }
            }

            // Execute the flow itself
            let flow = self
                .flows
                .remove(flow_id)
                .ok_or_else(|| anyhow::anyhow!("Flow not found: {}", flow_id))?;

            let context = flow.execute().await?;

            executing.remove(flow_id);
            executed.insert(flow_id.to_string());

            Ok(context)
        })
    }

    /// Execute flows in parallel where possible
    pub async fn execute_parallel(&mut self) -> Result<HashMap<String, FlowContext>> {
        // This is a simplified implementation
        // A full implementation would analyze the dependency graph
        // and execute independent flows in parallel
        self.execute_all().await
    }
}

impl Default for FlowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
