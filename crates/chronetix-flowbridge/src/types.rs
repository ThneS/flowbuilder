// SPDX-License-Identifier: Apache-2.0

//! Type definitions for Chronetix FlowBuilder bridge

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Envelope structure for message passing (aligned with Chronetix RFC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub header: EnvelopeHeader,
    pub payload: Vec<u8>,
}

/// Envelope header with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeHeader {
    pub message_id: String,
    pub timestamp: u64,
    pub content_type: String,
    pub schema_version: String,
    pub routing_key: Option<String>,
    pub priority: Option<u8>,
    pub deadline: Option<u64>,
    pub metadata: HashMap<String, String>,
}

/// Event bus interface for publish/subscribe communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    pub bus_type: BusType,
    pub topic_filters: Vec<String>,
    pub qos_policy: QosPolicy,
}

/// Type of event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusType {
    InProcess,
    Ipc,
    Network,
}

/// Quality of Service policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosPolicy {
    pub reliability: ReliabilityLevel,
    pub durability: DurabilityLevel,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

/// Reliability level for message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityLevel {
    BestEffort,
    Reliable,
    Guaranteed,
}

/// Durability level for message persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurabilityLevel {
    Volatile,
    Transient,
    Persistent,
}

/// Data port interface for high-throughput data streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPortConfig {
    pub port_type: PortType,
    pub buffer_size: usize,
    pub credit_system: CreditSystem,
    pub flow_control: FlowControlPolicy,
}

/// Type of data port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortType {
    InProcess,
    SharedMemory,
    Network,
}

/// Credit-based flow control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditSystem {
    pub initial_credits: u32,
    pub max_credits: u32,
    pub credit_threshold: u32,
}

/// Flow control policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowControlPolicy {
    pub backpressure_enabled: bool,
    pub rate_limit_ms: Option<u64>,
    pub burst_size: Option<u32>,
}
