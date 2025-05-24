use anyhow::Result;
use std::{future::Future, pin::Pin};

use crate::context::SharedContext;

type StepFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub type Step = Box<dyn FnOnce(SharedContext) -> StepFuture + Send>;
