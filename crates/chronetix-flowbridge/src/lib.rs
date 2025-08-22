// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "inproc")]
pub mod inproc;
pub mod types;

use anyhow::Result;
use serde::Deserialize;
use types::CompileOutput;

/// FlowAdapter: 从 Flowbuilder 的图/DSL 编译出 Chronetix 可执行描述
pub trait FlowAdapter {
    type GraphDef;

    fn compile(graph: &Self::GraphDef) -> Result<CompileOutput>;
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
