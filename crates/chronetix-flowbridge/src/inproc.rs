// SPDX-License-Identifier: Apache-2.0

use crate::types::{ChannelKind, PluginKind};
use crate::{contract as c, FlowAdapter, SimpleGraph};
use anyhow::{anyhow, Result};

pub struct InprocAdapter;

impl FlowAdapter for InprocAdapter {
    type GraphDef = SimpleGraph;

    fn compile(graph: &Self::GraphDef) -> Result<c::CompileOutput> {
        // 1) Manifests
        let manifests: Vec<c::PluginManifest> = graph
            .nodes
            .iter()
            .map(|n| {
                let kind = match n.impl_kind.as_str() {
                    "wasm" => PluginKind::Wasm,
                    "dylib" => PluginKind::Dylib,
                    "process" => PluginKind::Process,
                    other => {
                        tracing::warn!(
                            "unknown impl_kind: {other}, fallback to process"
                        );
                        PluginKind::Process
                    }
                };
                // Map to contract::PluginManifest (minimal)
                c::PluginManifest {
                    plugin_id: n.id.clone(),
                    version: Some("0.1.0".to_string()),
                    category: Some("Business".to_string()),
                    role: Some(
                        match kind {
                            PluginKind::Wasm
                            | PluginKind::Dylib
                            | PluginKind::Process => "Transform",
                        }
                        .to_string(),
                    ),
                    origin: Some("external".to_string()),
                    artifact: Some(c::Artifact {
                        kind: match kind {
                            PluginKind::Wasm => "WasmComponent",
                            PluginKind::Dylib => "Dylib",
                            PluginKind::Process => "Process",
                        }
                        .to_string(),
                        uri: n.entry.clone(),
                    }),
                    io: None,
                    qos: n.qos.clone().map(|q| serde_json::json!({ "qos": q })),
                    timers: None,
                    features: None,
                    annotations: None,
                }
            })
            .collect::<Vec<_>>();

        // 2) Routes
        let routes: Vec<c::Route> = graph
            .edges
            .iter()
            .map(|e| {
                let channel = match e.channel.as_str() {
                    "event-bus" => ChannelKind::EventBus,
                    "stream" => ChannelKind::Stream,
                    "blob-ref" => ChannelKind::BlobRef,
                    other => {
                        tracing::warn!(
                            "unknown channel: {other}, fallback to stream"
                        );
                        ChannelKind::Stream
                    }
                };
                // Map to contract::Route (assume data plane stream)
                c::Route {
                    from: e.from.clone(),
                    to: e.to.clone(),
                    plane: match channel {
                        ChannelKind::EventBus => "control",
                        _ => "data",
                    }
                    .to_string(),
                    topic: e.label.clone(),
                    port: if matches!(channel, ChannelKind::Stream) {
                        Some(format!("dataport/{}-{}", e.from, e.to))
                    } else {
                        None
                    },
                    content_type: "application/cbor".to_string(),
                    schema_ver: "v1".to_string(),
                    buffer: None,
                    watermark: None,
                }
            })
            .collect::<Vec<_>>();

        // 3) Schemas（最简：占位示例）
        let schemas = vec![c::SchemaDescriptor {
            content_type: "application/cbor".into(),
            schema_ver: "v1".into(),
            schema_ref: "registry://schemas/default@v1".into(),
        }];

        if manifests.is_empty() {
            return Err(anyhow!("no nodes in graph"));
        }

        Ok(c::CompileOutput {
            manifests,
            routes,
            schemas,
        })
    }
}
