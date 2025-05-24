use anyhow::{Result, anyhow};
use flowbuilder::{builder::FlowBuilder, context::SharedContext};
use std::time::Duration;

async fn run(_ctx: SharedContext) -> Result<()> {
    println!("running...");
    Ok(())
}

async fn check(ctx: SharedContext) -> Result<()> {
    println!("checking...");
    ctx.lock().await.ok = true;
    Ok(())
}

async fn stop(_ctx: SharedContext) -> Result<()> {
    println!("stopping...");
    Ok(())
}

async fn finish(_ctx: SharedContext) -> Result<()> {
    println!("finishing...");
    Ok(())
}

async fn finalize(_ctx: SharedContext) -> Result<()> {
    Err(anyhow!("Simulated finalize failure"))
}

async fn sub1(_ctx: SharedContext) -> Result<()> {
    println!("sub1");
    Ok(())
}

async fn sub2(_ctx: SharedContext) -> Result<()> {
    println!("sub2");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    FlowBuilder::new()
        .named_step("run", run)
        .named_step("check", check)
        .wait_until(|ctx| ctx.ok, Duration::from_secs(1), 3)
        .step_if(|ctx| ctx.ok, stop)
        .step_if(|ctx| ctx.ok, finish)
        .step_handle_error("finalize", finalize, |ctx, e| {
            ctx.errors.push(format!("{}", e));
            Ok(())
        })
        .subflow_if(
            |ctx| ctx.ok,
            || {
                FlowBuilder::new()
                    .named_step("sub1", sub1)
                    .named_step("sub2", sub2)
            },
        )
        .run_all()
        .await?;

    Ok(())
}
