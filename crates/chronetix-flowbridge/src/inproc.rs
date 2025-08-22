// SPDX-License-Identifier: Apache-2.0

//! In-process implementation of FlowAdapter and NodeRunner

use crate::{
    CompilationResult, FlowAdapter, FlowGraph, NodeInput, NodeOutput,
    NodeRunner, PluginManifest, Route, Schema,
};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// In-process flow adapter implementation
pub struct InProcFlowAdapter {
    config: InProcConfig,
}

/// Configuration for in-process adapter
#[derive(Debug, Clone)]
pub struct InProcConfig {
    pub enable_metrics: bool,
    pub buffer_size: usize,
}

impl Default for InProcConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            buffer_size: 1000,
        }
    }
}

impl InProcFlowAdapter {
    /// Create a new in-process flow adapter
    pub fn new(config: InProcConfig) -> Self {
        Self { config }
    }
}

impl FlowAdapter for InProcFlowAdapter {
    fn compile(&self, graph: &FlowGraph) -> Result<CompilationResult> {
        let mut manifests = Vec::new();
        let mut routes = Vec::new();
        let mut schemas = Vec::new();

        // Generate manifests for each node
        for node in &graph.nodes {
            let manifest = PluginManifest {
                name: node.id.clone(),
                version: "1.0.0".to_string(),
                inputs: vec!["input".to_string()],
                outputs: vec!["output".to_string()],
            };
            manifests.push(manifest);

            // Generate default schema for node
            let schema = Schema {
                content_type: "application/json".to_string(),
                schema_version: "1.0".to_string(),
                definition: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": { "type": "object" },
                        "metadata": { "type": "object" }
                    }
                }),
            };
            schemas.push(schema);
        }

        // Generate routes for each edge
        for edge in &graph.edges {
            let route = Route {
                topic: format!("{}_{}", edge.from, edge.to),
                from_port: edge.from.clone(),
                to_port: edge.to.clone(),
            };
            routes.push(route);
        }

        Ok(CompilationResult {
            manifests,
            routes,
            schemas,
        })
    }
}

/// In-process node runner implementation
pub struct InProcNodeRunner {
    node_id: String,
    processor: Box<dyn NodeProcessor + Send + Sync>,
}

/// Trait for processing node logic
pub trait NodeProcessor {
    fn process(&self, input: &NodeInput) -> Result<NodeOutput>;
}

/// Simple passthrough processor for testing
pub struct PassthroughProcessor;

impl NodeProcessor for PassthroughProcessor {
    fn process(&self, input: &NodeInput) -> Result<NodeOutput> {
        Ok(NodeOutput {
            data: input.data.clone(),
            metadata: input.metadata.clone(),
        })
    }
}

impl InProcNodeRunner {
    /// Create a new in-process node runner
    pub fn new(
        node_id: String,
        processor: Box<dyn NodeProcessor + Send + Sync>,
    ) -> Self {
        Self { node_id, processor }
    }

    /// Create a simple passthrough runner for testing
    pub fn passthrough(node_id: String) -> Self {
        Self::new(node_id, Box::new(PassthroughProcessor))
    }
}

impl NodeRunner for InProcNodeRunner {
    async fn run(&self, input: NodeInput) -> Result<NodeOutput> {
        // Simulate async processing
        tokio::task::yield_now().await;

        // Process the input
        self.processor.process(&input)
    }
}

/// In-process execution engine for running compiled flows
pub struct InProcExecutionEngine {
    runners: HashMap<String, InProcNodeRunner>,
    channels: HashMap<String, mpsc::Sender<NodeOutput>>,
}

impl InProcExecutionEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    /// Add a node runner
    pub fn add_runner(&mut self, node_id: String, runner: InProcNodeRunner) {
        self.runners.insert(node_id, runner);
    }

    /// Execute a flow graph
    pub async fn execute(
        &mut self,
        graph: &FlowGraph,
        input: NodeInput,
    ) -> Result<Vec<NodeOutput>> {
        let mut results = Vec::new();

        // Simple sequential execution for demo purposes
        for node in &graph.nodes {
            if let Some(runner) = self.runners.get(&node.id) {
                let output = runner.run(input.clone()).await?;
                results.push(output);
            }
        }

        Ok(results)
    }
}

impl Default for InProcExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FlowEdge, FlowNode};

    #[tokio::test]
    async fn test_inproc_adapter() {
        let adapter = InProcFlowAdapter::new(InProcConfig::default());

        let graph = FlowGraph {
            nodes: vec![FlowNode {
                id: "node1".to_string(),
                node_type: "transform".to_string(),
                config: serde_json::json!({}),
            }],
            edges: vec![],
        };

        let result = adapter.compile(&graph).expect("Failed to compile graph");
        assert_eq!(result.manifests.len(), 1);
        assert_eq!(result.manifests[0].name, "node1");
    }

    #[tokio::test]
    async fn test_inproc_runner() {
        let runner = InProcNodeRunner::passthrough("test_node".to_string());

        let input = NodeInput {
            data: serde_json::json!({"message": "hello"}),
            metadata: HashMap::new(),
        };

        let output =
            runner.run(input.clone()).await.expect("Failed to run node");
        assert_eq!(output.data, input.data);
    }
}
