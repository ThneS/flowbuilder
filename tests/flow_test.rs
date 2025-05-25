#[cfg(feature = "logger")]
use flowbuilder::prelude::Logger;
use flowbuilder::prelude::{FlowBuilder, SharedContext};

async fn run(_ctx: SharedContext) -> anyhow::Result<()> {
    #[cfg(feature = "logger")]
    Logger::new().info("run");
    #[cfg(not(feature = "logger"))]
    println!("run");
    Ok(())
}

async fn check(ctx: SharedContext) -> anyhow::Result<()> {
    ctx.lock().await.ok = true;
    Ok(())
}

async fn fail(_ctx: SharedContext) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("fail"))
}

#[tokio::test]
async fn test_flow_success() {
    let result = FlowBuilder::new()
        .named_step("run", run)
        .named_step("check", check)
        .step_if(|ctx| ctx.ok, run)
        .run_all()
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_flow_handle_error() {
    let result = FlowBuilder::new()
        .step_handle_error("fail", fail, |ctx, err| {
            ctx.errors.push(err.to_string());
            Ok(())
        })
        .step(|ctx| {
            Box::pin(async move {
                let guard = ctx.lock().await;
                assert_eq!(guard.errors.len(), 1);
                Ok(())
            })
        })
        .run_all()
        .await;

    assert!(result.is_ok());
}
