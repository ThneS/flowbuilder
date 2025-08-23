// SPDX-License-Identifier: Apache-2.0

//! Conversions from legacy `types` to new `contract` shapes.

use crate::{contract as c, types as t};

impl From<&t::PluginKind> for String {
    fn from(k: &t::PluginKind) -> Self {
        match k {
            t::PluginKind::Wasm => "WasmComponent".to_string(),
            t::PluginKind::Dylib => "Dylib".to_string(),
            t::PluginKind::Process => "Process".to_string(),
        }
    }
}

impl From<&t::PluginManifest> for c::PluginManifest {
    fn from(m: &t::PluginManifest) -> Self {
        c::PluginManifest {
            plugin_id: m.id.clone(),
            version: Some("0.1.0".to_string()),
            category: Some("Business".to_string()),
            role: Some("Transform".to_string()),
            origin: Some("external".to_string()),
            artifact: Some(c::Artifact {
                kind: String::from(&m.kind),
                uri: m.entry.clone(),
            }),
            io: None,
            qos: m.resources.as_ref().map(|r| {
                serde_json::json!({
                    "deadline_ns": r.deadline_ns,
                    "priority": r.priority,
                    "qos": r.qos,
                })
            }),
            timers: m.timers.as_ref().map(|vv| {
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
            }),
            features: None,
            annotations: None,
        }
    }
}

impl From<&t::ChannelKind> for String {
    fn from(k: &t::ChannelKind) -> Self {
        match k {
            t::ChannelKind::EventBus => "control".to_string(),
            _ => "data".to_string(),
        }
    }
}

impl From<&t::RouteSpec> for c::Route {
    fn from(r: &t::RouteSpec) -> Self {
        c::Route {
            from: r.from.clone(),
            to: r.to.clone(),
            plane: r.plane.clone().unwrap_or_else(|| String::from(&r.channel)),
            topic: r.topic_or_label.clone(),
            port: match r.channel {
                t::ChannelKind::Stream => {
                    Some(format!("dataport/{}-{}", r.from, r.to))
                }
                _ => None,
            },
            content_type: r
                .content_type
                .clone()
                .unwrap_or_else(|| "application/cbor".to_string()),
            schema_ver: r
                .schema_ver
                .clone()
                .unwrap_or_else(|| "v1".to_string()),
            buffer: r.buffer,
            watermark: r.watermark,
        }
    }
}

impl From<&t::SchemaDescriptor> for c::SchemaDescriptor {
    fn from(s: &t::SchemaDescriptor) -> Self {
        c::SchemaDescriptor {
            content_type: s.content_type.clone(),
            schema_ver: format!("v{}", s.version),
            schema_ref: format!("registry://schemas/{}@v{}", s.id, s.version),
        }
    }
}

impl From<&t::CompileOutput> for c::CompileOutput {
    fn from(co: &t::CompileOutput) -> Self {
        c::CompileOutput {
            manifests: co.manifests.iter().map(|m| m.into()).collect(),
            routes: co.routes.iter().map(|r| r.into()).collect(),
            schemas: co.schemas.iter().map(|s| s.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_to_contract_route_defaults() {
        let legacy = t::RouteSpec {
            from: "a".into(),
            to: "b".into(),
            channel: t::ChannelKind::Stream,
            topic_or_label: "flow/a-b".into(),
            plane: None,
            content_type: None,
            schema_ver: None,
            buffer: None,
            watermark: None,
        };
        let r = c::Route::from(&legacy);
        assert_eq!(r.plane, "data");
        assert_eq!(r.content_type, "application/cbor");
        assert_eq!(r.schema_ver, "v1");
        assert!(r.port.is_some());
    }

    #[test]
    fn legacy_to_contract_manifest_basic() {
        let legacy = t::PluginManifest {
            id: "node-1".into(),
            kind: t::PluginKind::Wasm,
            entry: "file://plugins/node1.wasm".into(),
            params: serde_json::json!({"k":"v"}),
            resources: Some(t::ResourceHints {
                cpu: None,
                mem: None,
                qos: Some("best-effort".into()),
                priority: Some(5),
                deadline_ns: Some(2_000_000),
            }),
            timers: Some(vec![t::TimerBinding {
                id: "tick".into(),
                schedule: t::TimerSchedule {
                    interval_ms: Some(1000),
                    rate_hz: None,
                    cron: None,
                    start_at: None,
                    align_to: Some("second".into()),
                    jitter_pct: None,
                },
                miss_policy: Some("coalesce".into()),
                deadline_ns: None,
                priority: None,
            }]),
            bus_subscriptions: None,
            bus_publications: None,
        };

        let m = c::PluginManifest::from(&legacy);
        assert_eq!(m.plugin_id, "node-1");
        assert_eq!(m.version.as_deref(), Some("0.1.0"));
        assert_eq!(m.category.as_deref(), Some("Business"));
        assert_eq!(m.role.as_deref(), Some("Transform"));
        assert_eq!(m.origin.as_deref(), Some("external"));
        let a = m.artifact.as_ref().expect("artifact");
        assert_eq!(a.kind, "WasmComponent");
        assert_eq!(a.uri, "file://plugins/node1.wasm");
        assert!(m.qos.is_some());
        assert!(m.timers.as_ref().map(|v| !v.is_empty()).unwrap_or(false));
    }

    #[test]
    fn legacy_to_contract_route_eventbus_plane() {
        let legacy = t::RouteSpec {
            from: "ctrl".into(),
            to: "worker".into(),
            channel: t::ChannelKind::EventBus,
            topic_or_label: "control/topic".into(),
            plane: None,
            content_type: None,
            schema_ver: None,
            buffer: None,
            watermark: None,
        };
        let r = c::Route::from(&legacy);
        assert_eq!(r.plane, "control");
        assert!(r.port.is_none());
    }

    #[test]
    fn legacy_to_contract_schema_descriptor() {
        let legacy = t::SchemaDescriptor {
            id: "foo".into(),
            content_type: "application/json".into(),
            version: 2,
            meta: serde_json::json!({}),
        };
        let s = c::SchemaDescriptor::from(&legacy);
        assert_eq!(s.content_type, "application/json");
        assert_eq!(s.schema_ver, "v2");
        assert_eq!(s.schema_ref, "registry://schemas/foo@v2");
    }
}
