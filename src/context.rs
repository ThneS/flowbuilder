use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default, Clone)]
pub struct FlowContext {
    pub ok: bool,
    pub errors: Vec<String>,
}

pub type SharedContext = Arc<Mutex<FlowContext>>;
