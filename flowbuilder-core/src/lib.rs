//! # FlowBuilder Core
//!
//! Core flow building functionality including FlowBuilder and Flow execution.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod flow_builder;
mod flow;
mod executor;

#[cfg(test)]
mod tests;

pub use flow_builder::{FlowBuilder, Step, StepFuture};
pub use flow::Flow;
pub use executor::FlowExecutor;

/// Prelude module for core functionality
pub mod prelude {
    pub use crate::{FlowBuilder, Flow, FlowExecutor, Step, StepFuture};
    pub use flowbuilder_context::{FlowContext, SharedContext};
}