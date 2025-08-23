// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompileOutput {
    pub manifests: Vec<PluginManifest>,
    pub routes: Vec<Route>,
    pub schemas: Vec<SchemaDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    #[serde(rename = "plugin_id")]
    pub plugin_id: String,
    pub version: Option<String>,
    pub category: Option<String>,
    pub role: Option<String>,
    pub origin: Option<String>, // internal | external
    pub artifact: Option<Artifact>,
    pub io: Option<IoSpec>,
    pub qos: Option<serde_json::Value>,
    pub timers: Option<Vec<TimerBinding>>, // for System timer
    pub features: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    pub kind: String, // e.g. WasmComponent
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoSpec {
    pub inputs: Option<Vec<IoType>>,
    pub outputs: Option<Vec<IoType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoType {
    pub content_type: String,
    pub schema_ver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerBinding {
    pub id: String,
    pub schedule: TimerSchedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miss_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub from: String,
    pub to: String,
    pub plane: String,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    pub content_type: String,
    pub schema_ver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDescriptor {
    pub content_type: String,
    pub schema_ver: String,
    pub schema_ref: String,
}
