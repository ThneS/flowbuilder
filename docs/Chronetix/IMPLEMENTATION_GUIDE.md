<!-- SPDX-License-Identifier: Apache-2.0 -->

# Chronetix FlowBuilder Implementation Guide

This guide provides detailed instructions for implementing the integration between FlowBuilder and Chronetix runtime systems.

## Overview

The Chronetix FlowBuilder bridge enables execution of FlowBuilder workflows within Chronetix's distributed runtime environment, providing:

- **Flow Compilation**: Convert FlowBuilder DAGs to Chronetix plugin manifests
- **Runtime Integration**: Execute nodes using Chronetix's executor framework  
- **Data Transport**: High-performance data streaming between nodes
- **Event Communication**: Publish/subscribe messaging for control flow
- **Observability**: Metrics and health monitoring integration

## Architecture

### Core Components

1. **FlowAdapter**: Compiles FlowBuilder graphs into Chronetix-compatible artifacts
2. **NodeRunner**: Executes individual nodes within Chronetix runtime
3. **EventBus**: Handles control plane messaging and coordination
4. **DataPort**: Manages high-throughput data streaming between nodes
5. **MetricsCollector**: Provides observability and monitoring capabilities

### Integration Patterns

#### In-Process Integration
- Direct memory sharing between FlowBuilder and Chronetix
- Lowest latency, highest throughput
- Single process deployment

#### IPC Integration  
- Inter-process communication using shared memory and message queues
- Process isolation with efficient data transfer
- Multi-process deployment on single machine

#### Network Integration
- TCP/UDP networking for distributed deployment
- Full geographic distribution capability
- Network-based service mesh integration

## Implementation Steps

### 1. Flow Compilation

Transform FlowBuilder workflow definitions into Chronetix artifacts:

```rust
use chronetix_flowbridge::{FlowAdapter, InProcFlowAdapter};

let adapter = InProcFlowAdapter::new(Default::default());
let result = adapter.compile(&flow_graph)?;

// result contains:
// - manifests: Plugin definitions for each node
// - routes: Topic routing configuration  
// - schemas: Data type definitions
```

### 2. Node Execution

Implement node logic using the NodeRunner trait:

```rust
use chronetix_flowbridge::{NodeRunner, InProcNodeRunner};

let runner = InProcNodeRunner::new(
    "transform_node".to_string(),
    Box::new(MyTransformProcessor)
);

let output = runner.run(input).await?;
```

### 3. Event Bus Setup

Configure publish/subscribe messaging:

```rust
// WIT interface definition
interface event-bus {
    publish: func(topic: string, event: event-message) -> result<_, error-info>;
    subscribe: func(topic: string, subscriber-id: entity-id) -> result<subscription-handle, error-info>;
}
```

### 4. Data Streaming

Set up high-throughput data ports:

```rust
// WIT interface definition  
interface data-stream {
    create-stream: func(config: stream-config) -> result<stream-handle, error-info>;
    send: func(handle: stream-handle, frame: data-frame) -> result<_, error-info>;
    receive: func(handle: stream-handle) -> result<option<data-frame>, error-info>;
}
```

### 5. Metrics Integration

Enable observability and monitoring:

```rust
// WIT interface definition
interface metrics-collector {
    record-counter: func(name: string, value: u64, labels: metadata) -> result<_, error-info>;
    record-gauge: func(name: string, value: f64, labels: metadata) -> result<_, error-info>;
    start-timer: func(name: string, labels: metadata) -> result<timer-handle, error-info>;
}
```

## Configuration

### FlowBuilder Configuration

Add Chronetix integration to your FlowBuilder configuration:

```yaml
chronetix:
  enabled: true
  runtime: "inproc"  # inproc | ipc | network
  event_bus:
    type: "inproc"
    buffer_size: 1000
  data_ports:
    type: "inproc"  
    flow_control: "credit_based"
  metrics:
    enabled: true
    export_format: "prometheus"
```

### Chronetix Configuration

Configure Chronetix to host FlowBuilder nodes:

```toml
[runtime]
plugin_discovery = ["flowbuilder"]
execution_model = "async"

[event_bus]
transport = "inproc"
topics = ["flowbuilder.*"]

[data_ports]
transport = "inproc"
credit_system = true
```

## Deployment Patterns

### Single Process (InProc)
- Both FlowBuilder and Chronetix in same process
- Shared memory, direct function calls
- Best performance, simplest deployment

### Multi-Process (IPC)
- FlowBuilder coordinator + Chronetix executors
- Shared memory + message queues
- Process isolation with good performance

### Distributed (Network)
- FlowBuilder on coordinator nodes
- Chronetix executors on worker nodes  
- Network protocols for all communication
- Maximum scalability and fault tolerance

## Performance Considerations

### Latency Optimization
- Use in-process transport for latency-critical paths
- Minimize serialization overhead with binary protocols
- Batch small messages to reduce round-trips

### Throughput Optimization  
- Enable credit-based flow control for backpressure
- Use parallel execution where possible
- Configure appropriate buffer sizes

### Memory Management
- Pool and reuse data frames to reduce allocations
- Use zero-copy transfers where supported
- Monitor memory usage and implement cleanup policies

## Error Handling

### Failure Recovery
- Implement retry logic with exponential backoff
- Use circuit breakers for external dependencies
- Provide graceful degradation modes

### Diagnostics
- Comprehensive logging at debug/trace levels
- Structured metrics for key performance indicators
- Health checks for all major components

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock external dependencies
- Verify error handling paths

### Integration Tests  
- Test complete flow execution
- Verify cross-component communication
- Test failure and recovery scenarios

### Performance Tests
- Benchmark throughput and latency
- Test under various load conditions
- Verify resource usage patterns

## Security Considerations

### Process Isolation
- Run untrusted nodes in separate processes
- Use sandboxing techniques where available
- Implement resource limits and quotas

### Data Security
- Encrypt sensitive data in transit and at rest
- Implement access controls for data flows
- Audit trail for data access and modifications

### Network Security
- Use TLS for network communications
- Implement authentication and authorization
- Network segmentation for sensitive workloads

## Troubleshooting

### Common Issues

1. **High Latency**: Check transport configuration, consider in-process mode
2. **Memory Leaks**: Verify proper cleanup of resources and data frames
3. **Deadlocks**: Review flow control and backpressure settings
4. **Data Loss**: Check reliability settings and error handling

### Debugging Tools

- Built-in metrics dashboards
- Trace logging with correlation IDs  
- Flow execution visualization
- Performance profiling integration

## Migration Guide

### From Pure FlowBuilder
1. Add chronetix-flowbridge dependency
2. Update workflow definitions with Chronetix annotations
3. Configure runtime selection (inproc/ipc/network)
4. Test and validate performance characteristics

### To Full Chronetix
1. Implement custom NodeProcessor for business logic
2. Migrate to native Chronetix plugin format
3. Update deployment and configuration
4. Validate functional and performance requirements

## References

- [FlowBuilder Documentation](../README.md)
- [Chronetix Architecture](./FLOWBUILDER_INTEGRATION.md)
- [WIT Interface Specifications](../../wit/flow/)
- [Performance Benchmarks](./benchmarks.md)
- [Security Guidelines](./security.md)