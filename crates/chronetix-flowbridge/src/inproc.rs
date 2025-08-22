// SPDX-License-Identifier: Apache-2.0

use crate::types::*;
use crate::{FlowAdapter, SimpleGraph};
use anyhow::{anyhow, Result};

pub struct InprocAdapter;

impl FlowAdapter for InprocAdapter {
    type GraphDef = SimpleGraph;

    fn compile(graph: &Self::GraphDef) -> Result<CompileOutput> {
        // 1) Manifests
        let manifests = graph
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

                PluginManifest {
                    id: n.id.clone(),
                    kind,
                    entry: n.entry.clone(),
                    params: serde_json::json!({}),
                    resources: Some(ResourceHints {
                        cpu: None,
                        mem: None,
                        qos: n.qos.clone(),
                        priority: n.priority,
                        deadline_ns: n.deadline_ns,
                    }),
                    timers: None, // TODO: extract from node definition
                    bus_subscriptions: None, // TODO: extract from node definition
                    bus_publications: None, // TODO: extract from node definition
                }
            })
            .collect::<Vec<_>>();

        // 2) Routes
        let routes = graph
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

                RouteSpec {
                    from: e.from.clone(),
                    to: e.to.clone(),
                    channel,
                    topic_or_label: e.label.clone(),
                    plane: Some("data".to_string()), // Default to data plane
                    content_type: Some("application/cbor".to_string()), // Default
                    schema_ver: Some("v1".to_string()), // Default
                    buffer: None,
                    watermark: None,
                }
            })
            .collect::<Vec<_>>();

        // 3) Schemas（最简：占位示例）
        let schemas = vec![SchemaDescriptor {
            id: "default-control".into(),
            content_type: "application/cbor".into(),
            version: 1,
            meta: serde_json::json!({}),
        }];

        if manifests.is_empty() {
            return Err(anyhow!("no nodes in graph"));
        }

        Ok(CompileOutput {
            manifests,
            routes,
            schemas,
        })
    }
}
