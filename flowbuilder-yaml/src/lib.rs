//! # FlowBuilder YAML
//!
//! Dynamic flow construction from YAML/JSON configuration

mod config;
mod executor;
mod expression;
mod loader;
mod parser;
mod runtime_integration;

pub use config::*;
pub use executor::*;
pub use expression::*;
pub use loader::*;
pub use parser::*;
pub use runtime_integration::*;

/// Prelude module for YAML functionality
pub mod prelude {
    pub use crate::{
        DynamicFlowExecutor, ExpressionEvaluator, WorkflowConfig, WorkflowLoader, YamlFlowBuilder,
        YamlRuntimeIntegrator,
    };
}
