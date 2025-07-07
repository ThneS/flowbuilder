use flowbuilder::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== FlowBuilder Modular Architecture Demo ===\n");

    // Basic flow example
    println!("1. Basic Flow Example:");
    let basic_flow = FlowBuilder::new()
        .step(|_ctx| async move {
            println!("  Step 1: Basic step executed");
            Ok(())
        })
        .named_step("step_2", |_ctx| async move {
            println!("  Step 2: Named step executed");
            Ok(())
        })
        .step_if(
            |_ctx| true, // Always execute
            |_ctx| async move {
                println!("  Step 3: Conditional step executed");
                Ok(())
            },
        );

    let result = basic_flow.execute().await?;
    result.print_summary();

    println!("=== Demo Completed ===");
    Ok(())
}
