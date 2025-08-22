// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub kind: PluginKind, // wasm/dylib/process
    pub entry: String,    // 路径或标识
    pub params: serde_json::Value,
    pub resources: Option<ResourceHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginKind {
    Wasm,
    Dylib,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHints {
    pub cpu: Option<String>,
    pub mem: Option<String>,
    pub qos: Option<String>,
    pub priority: Option<u8>,
    pub deadline_ns: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSpec {
    pub from: String,           // 节点/端口
    pub to: String,             // 节点/端口
    pub channel: ChannelKind,   // event-bus / stream / blob-ref
    pub topic_or_label: String, // topic label 或 stream label
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelKind {
    EventBus,
    Stream,
    BlobRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDescriptor {
    pub id: String,
    pub content_type: String, // application/cbor / application/arrow+ipc ...
    pub version: u32,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOutput {
    pub manifests: Vec<PluginManifest>,
    pub routes: Vec<RouteSpec>,
    pub schemas: Vec<SchemaDescriptor>,
}
