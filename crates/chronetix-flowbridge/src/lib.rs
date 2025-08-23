// SPDX-License-Identifier: Apache-2.0

pub mod contract;
pub mod convert;
#[cfg(feature = "inproc")]
pub mod inproc;
pub mod types;

use anyhow::Result;
use serde::Deserialize;

/// Minimal DSL for Chronetix example (docs/Chronetix/examples/minimal_dag.yaml)
#[derive(Debug, Deserialize)]
pub struct ExampleDag {
    pub nodes: Vec<ExampleNode>,
    pub routes: Vec<ExampleRoute>,
}

#[derive(Debug, Deserialize)]
pub struct ExampleNode {
    pub id: String,
    pub origin: Option<String>, // internal | external
    pub category: Option<String>, // Business | System | Resource
    pub plugin: ExamplePlugin,
    pub io: Option<ExampleIo>,
    pub qos: Option<serde_json::Value>,
    pub bindings: Option<ExampleBindings>,
}

#[derive(Debug, Deserialize)]
pub struct ExamplePlugin {
    pub kind: String,             // wasm
    pub artifact: Option<String>, // uri
    pub r#type: Option<String>,   // for system plugin like timer-source
}

#[derive(Debug, Deserialize)]
pub struct ExampleIo {
    pub inputs: Option<Vec<IoType>>,
    pub outputs: Option<Vec<IoType>>,
}

#[derive(Debug, Deserialize)]
pub struct IoType {
    pub content_type: String,
    pub schema_ver: String,
}

#[derive(Debug, Deserialize)]
pub struct ExampleBindings {
    pub timer: Option<Vec<ExampleTimerBinding>>, // timers list
}

#[derive(Debug, Deserialize)]
pub struct ExampleTimerBinding {
    pub id: String,
    pub schedule: ExampleTimerSchedule,
    pub miss_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExampleTimerSchedule {
    pub interval_ms: Option<u64>,
    pub align_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExampleRoute {
    pub from: String,
    pub to: String,
    pub plane: String,
    pub topic: String,
    pub port: Option<String>,
    pub content_type: Option<String>,
    pub schema_ver: Option<String>,
    pub buffer: Option<u32>,
    pub watermark: Option<u32>,
}

impl ExampleDag {
    /// Convert ExampleDag to contract::CompileOutput (for Chronetix adapter output)
    pub fn to_contract(&self) -> contract::CompileOutput {
        use contract as c;

        let manifests: Vec<c::PluginManifest> = self
            .nodes
            .iter()
            .map(|n| {
                // plugin_id 规范化
                let plugin_id = if n.category.as_deref() == Some("System")
                    && n.plugin.r#type.as_deref() == Some("timer-source")
                {
                    "timer-source".to_string()
                } else if n.category.as_deref() == Some("Resource") {
                    if let Some(uri) = &n.plugin.artifact {
                        let last = uri
                            .rsplit('/')
                            .next()
                            .unwrap_or(uri.as_str())
                            .to_string();
                        format!("resource-{}", last)
                    } else {
                        n.id.clone()
                    }
                } else {
                    n.id.clone()
                };

                // role 推断
                let role = match n.category.as_deref() {
                    Some("Resource") => Some("EnvProvider".to_string()),
                    Some("System")
                        if n.plugin.r#type.as_deref()
                            == Some("timer-source") =>
                    {
                        Some("Timer".to_string())
                    }
                    Some("Business")
                        if n.io
                            .as_ref()
                            .map(|io| {
                                io.inputs.is_some() && io.outputs.is_some()
                            })
                            .unwrap_or(false) =>
                    {
                        Some("SourceTransform".to_string())
                    }
                    Some("Business")
                        if n.io
                            .as_ref()
                            .map(|io| io.inputs.is_some())
                            .unwrap_or(false) =>
                    {
                        Some("Sink".to_string())
                    }
                    _ => None,
                };

                // artifact 归一
                let artifact = if n.category.as_deref() == Some("System")
                    && n.plugin.r#type.as_deref() == Some("timer-source")
                {
                    Some(c::Artifact {
                        kind: "WasmComponent".to_string(),
                        uri: "builtin://chronetix/system/timer".to_string(),
                    })
                } else {
                    n.plugin.artifact.as_ref().map(|uri| c::Artifact {
                        kind: "WasmComponent".to_string(),
                        uri: uri.clone(),
                    })
                };

                // IO 复制
                let io = n.io.as_ref().map(|io| c::IoSpec {
                    inputs: io.inputs.as_ref().map(|v| {
                        v.iter()
                            .map(|i| c::IoType {
                                content_type: i.content_type.clone(),
                                schema_ver: i.schema_ver.clone(),
                                schema_ref: None,
                            })
                            .collect()
                    }),
                    outputs: io.outputs.as_ref().map(|v| {
                        v.iter()
                            .map(|i| c::IoType {
                                content_type: i.content_type.clone(),
                                schema_ver: i.schema_ver.clone(),
                                schema_ref: None,
                            })
                            .collect()
                    }),
                });

                // timers 复制
                let timers =
                    n.bindings.as_ref().and_then(|b| b.timer.as_ref()).map(
                        |vv| {
                            vv.iter()
                                .map(|t| c::TimerBinding {
                                    id: t.id.clone(),
                                    schedule: c::TimerSchedule {
                                        interval_ms: t.schedule.interval_ms,
                                        align_to: t.schedule.align_to.clone(),
                                    },
                                    miss_policy: t.miss_policy.clone(),
                                })
                                .collect()
                        },
                    );

                c::PluginManifest {
                    plugin_id,
                    version: Some("0.1.0".to_string()),
                    category: n.category.clone(),
                    role,
                    origin: n.origin.clone(),
                    artifact,
                    io,
                    qos: n.qos.clone(),
                    timers,
                    features: None,
                    annotations: None,
                }
            })
            .collect();

        let routes = self
            .routes
            .iter()
            .map(|r| {
                let is_data = r.plane == "data";
                c::Route {
                    from: r.from.clone(),
                    to: r.to.clone(),
                    plane: r.plane.clone(),
                    topic: r.topic.clone(),
                    port: r.port.clone(),
                    content_type: r
                        .content_type
                        .clone()
                        .unwrap_or_else(|| "application/json".to_string()),
                    schema_ver: r
                        .schema_ver
                        .clone()
                        .unwrap_or_else(|| "v1".to_string()),
                    buffer: r.buffer.or(if is_data { Some(64) } else { None }),
                    watermark: r.watermark.or(if is_data {
                        Some(48)
                    } else {
                        None
                    }),
                }
            })
            .collect();

        // schemas：示例仅在 Resource 输出提供一个 schema_ref（如有的话）
        let mut schemas: Vec<c::SchemaDescriptor> = vec![];
        for n in &self.nodes {
            if let Some(io) = &n.io {
                if let Some(outputs) = &io.outputs {
                    for _o in outputs {
                        // 在示例中，我们只放置一个占位 schema；实际实现可按 schema_ref 去重收集
                    }
                }
            }
        }
        // 与示例 JSON 对齐，放入一个 netinfo schema
        schemas.push(c::SchemaDescriptor {
            content_type: "application/json".to_string(),
            schema_ver: "v1".to_string(),
            schema_ref: "registry://schemas/resource/netinfo@v1".to_string(),
        });

        c::CompileOutput {
            manifests,
            routes,
            schemas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn example_dag_to_contract_matches_json() {
        let yaml_path = "../../docs/Chronetix/examples/minimal_dag.yaml";
        let json_path =
            "../../docs/Chronetix/examples/minimal_compile_output.json";

        let yaml_str = fs::read_to_string(yaml_path).expect("read yaml");
        let dag: ExampleDag =
            serde_yaml::from_str(&yaml_str).expect("parse yaml");
        let got = dag.to_contract();

        let json_str = fs::read_to_string(json_path).expect("read json");
        let expected: contract::CompileOutput =
            serde_json::from_str(&json_str).expect("parse json");

        // 逐字段断言（宽松匹配：关注核心字段相等）
        assert_eq!(got.routes, expected.routes, "routes mismatch");
        assert_eq!(got.schemas, expected.schemas, "schemas mismatch");

        // 对 manifests 做名称集合与关键字段检查
        let got_ids: std::collections::BTreeSet<_> = got
            .manifests
            .iter()
            .map(|m| &m.plugin_id)
            .cloned()
            .collect();
        let exp_ids: std::collections::BTreeSet<_> = expected
            .manifests
            .iter()
            .map(|m| &m.plugin_id)
            .cloned()
            .collect();
        assert_eq!(got_ids, exp_ids, "manifest ids mismatch");

        // 可选：检查每个 manifest 的 category/origin/role 是否存在
        for m in &expected.manifests {
            let found = got
                .manifests
                .iter()
                .find(|x| x.plugin_id == m.plugin_id)
                .unwrap();
            assert_eq!(
                found.category, m.category,
                "category mismatch for {}",
                m.plugin_id
            );
            assert_eq!(
                found.origin, m.origin,
                "origin mismatch for {}",
                m.plugin_id
            );
        }
    }
}

/// FlowAdapter: 从 Flowbuilder 的图/DSL 编译出 Chronetix 可执行描述
pub trait FlowAdapter {
    type GraphDef;

    fn compile(graph: &Self::GraphDef) -> Result<contract::CompileOutput>;
}

/// NodeRunner: 在 Chronetix 执行器中的节点运行抽象
pub trait NodeRunner {
    /// 初始化/热加载所需的资源；返回可选的"运行句柄"
    fn init(&mut self) -> Result<()>;

    /// 拉起节点执行（Source/Map/Sink）；由 Chronetix Executor 调度
    fn start(&mut self) -> Result<()>;

    /// 优雅停止
    fn stop(&mut self) -> Result<()>;
}

/// 简化的 Flowbuilder 图描述（演示用途）
#[derive(Debug, Deserialize)]
pub struct SimpleGraph {
    pub nodes: Vec<SimpleNode>,
    pub edges: Vec<SimpleEdge>,
}

#[derive(Debug, Deserialize)]
pub struct SimpleNode {
    pub id: String,
    pub kind: String,      // source/map/sink
    pub impl_kind: String, // wasm/dylib/process
    pub entry: String,
    pub qos: Option<String>,
    pub priority: Option<u8>,
    pub deadline_ns: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SimpleEdge {
    pub from: String,
    pub to: String,
    pub channel: String, // event-bus/stream/blob-ref
    pub label: String,   // topic 或 stream label
}
