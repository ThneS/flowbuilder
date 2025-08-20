use flowbuilder::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flowbuilder::logging::init();
    info!("=== FlowBuilder Modular Architecture Demo ===");

    // Basic flow example
    info!("1. Basic Flow Example:");
    let basic_flow = FlowBuilder::new()
        .step(|_ctx| async move {
            info!("  Step 1: Basic step executed");
            Ok(())
        })
        .named_step("step_2", |_ctx| async move {
            info!("  Step 2: Named step executed");
            Ok(())
        })
        .step_if(
            |_ctx| true, // Always execute
            |_ctx| async move {
                info!("  Step 3: Conditional step executed");
                Ok(())
            },
        );

    let result = basic_flow.execute().await?;
    result.print_summary();
    info!("=== Demo Completed ===");
    Ok(())
}
