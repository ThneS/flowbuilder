// SPDX-License-Identifier: Apache-2.0

//! Chronetix FlowBuilder Bridge
//!
//! This crate provides integration between FlowBuilder's DAG execution model
//! and Chronetix's distributed runtime systems.

pub mod types;

#[cfg(feature = "inproc")]
pub mod inproc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Flow adapter trait for converting FlowBuilder graphs to Chronetix manifests
pub trait FlowAdapter {
    /// Compile a flow graph into plugin manifests, routes, and schemas
    fn compile(&self, graph: &FlowGraph) -> Result<CompilationResult>;
}

/// Node runner trait for executing individual nodes in Chronetix runtime
pub trait NodeRunner {
    /// Run a node with given input and return output
    async fn run(&self, input: NodeInput) -> Result<NodeOutput>;
}

/// Simplified flow graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

/// Flow node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub node_type: String,
    pub config: serde_json::Value,
}

/// Flow edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub port: Option<String>,
}

/// Node input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInput {
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Node output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Compilation result containing manifests, routes, and schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub manifests: Vec<PluginManifest>,
    pub routes: Vec<Route>,
    pub schemas: Vec<Schema>,
}

/// Plugin manifest for Chronetix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub topic: String,
    pub from_port: String,
    pub to_port: String,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub content_type: String,
    pub schema_version: String,
    pub definition: serde_json::Value,
}
