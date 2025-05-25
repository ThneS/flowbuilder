#![cfg_attr(docsrs, feature(doc_cfg))]

mod context;
mod core;

#[cfg(feature = "logger")]
pub mod logger;

pub mod prelude {
    pub use crate::context::{FlowContext, SharedContext};
    pub use crate::core::*;
    #[cfg(feature = "logger")]
    pub use crate::logger::Logger;
}
