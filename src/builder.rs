use crate::{
    context::{FlowContext, SharedContext},
    step::Step,
};
use anyhow::{Result, anyhow};
use std::{future::Future, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub struct FlowBuilder {
    pub steps: Vec<Step>,
}

impl Default for FlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn step<Fut, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| Box::pin(f(ctx))));
        self
    }

    pub fn named_step<Fut, F>(mut self, name: &'static str, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                println!("[step:{}] running...", name);
                let result = f(ctx2).await;
                if let Err(ref e) = result {
                    println!("[step:{}] failed: {}", name, e);
                }
                result
            })
        }));
        self
    }

    pub fn step_if<Fut, F, Cond>(mut self, cond: Cond, mut f: F) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let guard = ctx2.lock().await;
                if cond(&guard) {
                    drop(guard);
                    f(ctx2).await
                } else {
                    println!("[step_if] condition not met, skipping step");
                    Ok(())
                }
            })
        }));
        self
    }

    pub fn wait_until<Cond>(mut self, cond: Cond, interval: Duration, max_retry: usize) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                for attempt in 0..max_retry {
                    {
                        let guard = ctx.lock().await;
                        if cond(&guard) {
                            println!("[wait_until] condition met on attempt {}", attempt + 1);
                            return Ok(());
                        }
                    }
                    println!(
                        "[wait_until] attempt {}/{} failed, waiting...",
                        attempt + 1,
                        max_retry
                    );
                    tokio::time::sleep(interval).await;
                }
                Err(anyhow!(
                    "wait_until: condition not met after {} retries",
                    max_retry
                ))
            })
        }));
        self
    }

    pub fn step_handle_error<Fut, F, H>(
        mut self,
        name: &'static str,
        mut f: F,
        mut handle: H,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        H: FnMut(&mut FlowContext, anyhow::Error) -> Result<()> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                println!("[step:{}] running...", name);
                match f(ctx2.clone()).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        println!("[step:{}] error: {}", name, e);
                        let mut guard = ctx2.lock().await;
                        handle(&mut guard, e)
                    }
                }
            })
        }));
        self
    }

    pub fn subflow_if<Cond, G>(mut self, cond: Cond, subflow_gen: G) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        G: Fn() -> FlowBuilder + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let guard = ctx2.lock().await;
                if cond(&guard) {
                    drop(guard);
                    let subflow = subflow_gen();
                    subflow.run_all_with_context(ctx2.clone()).await
                } else {
                    Ok(())
                }
            })
        }));
        self
    }

    pub async fn run_all(self) -> Result<()> {
        self.run_all_with_context(Arc::new(Mutex::new(FlowContext::default())))
            .await
    }

    pub async fn run_all_with_context(self, ctx: SharedContext) -> Result<()> {
        for step in self.steps {
            step(ctx.clone()).await?;
        }
        Ok(())
    }
}
