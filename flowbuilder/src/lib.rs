//! # FlowBuilder - Async Flow Orchestration Framework
//!
//! FlowBuilder is a modular async flow orchestration framework with conditional execution and context sharing.
//!
//! ## Features
//!
//! - `core` (default): Core flow building functionality
//! - `macros`: Procedural macros for easier flow definition
//! - `logger`: Tracing and logging support
//! - `runtime`: Advanced runtime features
//! - `full`: All features enabled
//!
//! ## Quick Start
//!
//! ```rust
//! use flowbuilder::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let flow = FlowBuilder::new()
//!         .step(|ctx| async move {
//!             println!("Step 1 executed");
//!             Ok(())
//!         })
//!         .step(|ctx| async move {
//!             println!("Step 2 executed");
//!             Ok(())
//!         })
//!         .build();
//!
//!     flow.execute().await?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export core functionality
pub use flowbuilder_context as context;
pub use flowbuilder_core::*;

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub use flowbuilder_runtime as runtime;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub use flowbuilder_yaml as yaml;

/// Prelude module for easy imports
pub mod prelude {
    pub use flowbuilder_context::{FlowContext, SharedContext};
    pub use flowbuilder_core::prelude::*;

    #[cfg(feature = "runtime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
    pub use flowbuilder_runtime::{
        EnhancedFlowOrchestrator, EnhancedOrchestratorConfig,
        EnhancedTaskExecutor, ExecutionComplexity, ExecutionResult,
        ExecutorConfig, NodeResult, PhaseResult,
    };

    // 细粒度子特性透传（仅当 runtime 启用且对应子特性启用）
    #[cfg(all(feature = "runtime", feature = "perf-metrics"))]
    pub use flowbuilder_runtime::ExecutionStats;

    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    pub use flowbuilder_yaml::prelude::*;
}
