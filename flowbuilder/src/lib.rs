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
//! use tracing::info;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 统一日志初始化（支持 RUST_LOG / FB_LOG_FORMAT）
//!     flowbuilder::logging::init();
//!     let flow = FlowBuilder::new()
//!         .step(|ctx| async move {
//!             info!("Step 1 executed");
//!             Ok(())
//!         })
//!         .step(|ctx| async move {
//!             info!("Step 2 executed");
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

/// 统一日志初始化工具
pub mod logging {
    use tracing_subscriber::EnvFilter;

    /// 初始化 tracing，支持 RUST_LOG 与 FB_LOG_FORMAT（pretty|compact|json）
    pub fn init() {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        match std::env::var("FB_LOG_FORMAT").as_deref() {
            Ok("json") => {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .json()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .init();
            }
            Ok("compact") => {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .compact()
                    .init();
            }
            _ => {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .pretty()
                    .init();
            }
        }
    }
}
