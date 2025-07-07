//! # FlowBuilder Core
//!
//! Core flow building functionality including FlowBuilder and Flow execution.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod executor;
mod flow;
mod flow_builder;

#[cfg(test)]
mod tests;

pub use executor::FlowExecutor;
pub use flow::Flow;
pub use flow_builder::{FlowBuilder, Step, StepFuture};

/// Prelude module for core functionality
pub mod prelude {
    pub use crate::{Flow, FlowBuilder, FlowExecutor, Step, StepFuture};
    pub use flowbuilder_context::{FlowContext, SharedContext};
}
