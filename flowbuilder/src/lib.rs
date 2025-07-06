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

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use flowbuilder_macros::*;

#[cfg(feature = "logger")]
#[cfg_attr(docsrs, doc(cfg(feature = "logger")))]
pub use flowbuilder_logger as logger;

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

    #[cfg(feature = "macros")]
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    pub use flowbuilder_macros::*;

    #[cfg(feature = "logger")]
    #[cfg_attr(docsrs, doc(cfg(feature = "logger")))]
    pub use flowbuilder_logger::Logger;

    #[cfg(feature = "runtime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
    pub use flowbuilder_runtime::*;

    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    pub use flowbuilder_yaml::prelude::*;
}
