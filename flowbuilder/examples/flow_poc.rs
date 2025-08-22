use anyhow::Result;
use chronetix_flowbridge::{
    FlowAdapter, FlowEdge, FlowGraph, FlowNode,
};
use chronetix_flowbridge::inproc::{InProcConfig, InProcFlowAdapter};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== FlowBuilder Chronetix PoC ===");

    // Create a simple DAG with transform nodes
    let flow_graph = FlowGraph {
        nodes: vec![
            FlowNode {
                id: "source".to_string(),
                node_type: "data_source".to_string(),
                config: serde_json::json!({
                    "type": "generator",
                    "rate": 10
                }),
            },
            FlowNode {
                id: "transform".to_string(),
                node_type: "transformer".to_string(),
                config: serde_json::json!({
                    "operation": "map",
                    "function": "uppercase"
                }),
            },
            FlowNode {
                id: "sink".to_string(),
                node_type: "data_sink".to_string(),
                config: serde_json::json!({
                    "destination": "console",
                    "format": "json"
                }),
            },
        ],
        edges: vec![
            FlowEdge {
                from: "source".to_string(),
                to: "transform".to_string(),
                port: Some("output".to_string()),
            },
            FlowEdge {
                from: "transform".to_string(),
                to: "sink".to_string(),
                port: Some("output".to_string()),
            },
        ],
    };

    // Create flow adapter and compile the graph
    let adapter = InProcFlowAdapter::new(InProcConfig::default());
    let result = adapter.compile(&flow_graph)?;

    // Print the compilation results
    println!("\n=== Plugin Manifests ===");
    for (i, manifest) in result.manifests.iter().enumerate() {
        println!("Manifest {}:", i + 1);
        println!("  Name: {}", manifest.name);
        println!("  Version: {}", manifest.version);
        println!("  Inputs: {:?}", manifest.inputs);
        println!("  Outputs: {:?}", manifest.outputs);
    }

    println!("\n=== Routes ===");
    for (i, route) in result.routes.iter().enumerate() {
        println!("Route {}:", i + 1);
        println!("  Topic: {}", route.topic);
        println!("  From Port: {}", route.from_port);
        println!("  To Port: {}", route.to_port);
    }

    println!("\n=== Schemas ===");
    for (i, schema) in result.schemas.iter().enumerate() {
        println!("Schema {}:", i + 1);
        println!("  Content Type: {}", schema.content_type);
        println!("  Schema Version: {}", schema.schema_version);
        println!(
            "  Definition: {}",
            serde_json::to_string_pretty(&schema.definition)?
        );
    }

    println!("\n=== PoC Completed Successfully ===");
    Ok(())
}
