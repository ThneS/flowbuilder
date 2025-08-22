// SPDX-License-Identifier: Apache-2.0

//! 最小 PoC：读取简化 DAG（内联 JSON），编译为 PluginManifest + routes + schemas（inproc）

use anyhow::Result;
use chronetix_flowbridge::{inproc::InprocAdapter, FlowAdapter, SimpleGraph};

fn main() -> Result<()> {
    let graph_json = r#"
    {
      "nodes": [
        {"id": "src", "kind": "source", "impl_kind": "wasm", "entry": "wasm://http_source", "qos":"normal", "priority": 5, "deadline_ns": 5000000},
        {"id": "map", "kind": "map",    "impl_kind": "wasm", "entry": "wasm://arrow_map",    "qos":"high",   "priority": 3, "deadline_ns": 4000000},
        {"id": "sink","kind": "sink",   "impl_kind": "wasm", "entry": "wasm://ftp_sink",     "qos":"normal", "priority": 6, "deadline_ns": 7000000}
      ],
      "edges": [
        {"from":"src","to":"map","channel":"stream","label":"dp.data.tx"},
        {"from":"map","to":"sink","channel":"stream","label":"dp.data.tx2"}
      ]
    }"#;

    let graph: SimpleGraph = serde_json::from_str(graph_json)?;
    let output = InprocAdapter::compile(&graph)?;

    println!("manifests = {:#?}", output.manifests);
    println!("routes = {:#?}", output.routes);
    println!("schemas = {:#?}", output.schemas);

    Ok(())
}
