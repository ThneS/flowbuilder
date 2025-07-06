//! # FlowBuilder YAML
//!
//! Dynamic flow construction from YAML/JSON configuration

mod config;
mod loader;
mod parser;
mod executor;
mod expression;

pub use config::*;
pub use loader::*;
pub use parser::*;
pub use executor::*;
pub use expression::*;

/// Prelude module for YAML functionality
pub mod prelude {
    pub use crate::{
        WorkflowConfig, WorkflowLoader, YamlFlowBuilder,
        DynamicFlowExecutor, ExpressionEvaluator
    };
}
