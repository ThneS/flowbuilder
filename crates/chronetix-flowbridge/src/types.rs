// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub kind: PluginKind, // wasm/dylib/process
    pub entry: String,    // 路径或标识
    pub params: serde_json::Value,
    pub resources: Option<ResourceHints>,
    pub timers: Option<Vec<TimerBinding>>,
    pub bus_subscriptions: Option<Vec<BusSubscription>>,
    pub bus_publications: Option<Vec<BusPublication>>,
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
pub struct TimerBinding {
    pub id: String,
    pub schedule: TimerSchedule,
    pub miss_policy: Option<String>, // skip | catch_up | coalesce
    pub deadline_ns: Option<u64>,
    pub priority: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSchedule {
    pub interval_ms: Option<u64>,
    pub rate_hz: Option<f64>,
    pub cron: Option<String>,
    pub start_at: Option<String>,
    pub align_to: Option<String>,
    pub jitter_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusSubscription {
    pub pattern: String,
    pub delivery: Option<String>, // at_least_once
    pub codec: Option<String>,    // json | cbor
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusPublication {
    pub topic: String,
    pub codec: Option<String>,              // json | cbor
    pub default_envelope: Option<serde_json::Value>, // {priority, deadline}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSpec {
    pub from: String,              // 节点/端口
    pub to: String,                // 节点/端口
    pub channel: ChannelKind,      // event-bus / stream / blob-ref
    pub topic_or_label: String,    // topic label 或 stream label
    pub plane: Option<String>,     // data | control
    pub content_type: Option<String>, // application/arrow-ipc etc.
    pub schema_ver: Option<String>,   // v1 etc.
    pub buffer: Option<u32>,       // buffer size
    pub watermark: Option<u32>,    // watermark threshold
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
