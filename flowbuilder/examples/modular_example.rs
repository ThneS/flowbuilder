use flowbuilder::prelude::*;

#[cfg(feature = "macros")]
use flowbuilder_macros::{named_step, step};

#[cfg(feature = "runtime")]
use flowbuilder_runtime::{FlowBuilderExt, ScheduleOptions};

#[cfg(feature = "logger")]
use flowbuilder_logger::Logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging if available
    #[cfg(feature = "logger")]
    Logger::init_tracing();

    println!("=== FlowBuilder Modular Architecture Demo ===\n");

    // Basic flow example
    println!("1. Basic Flow Example:");
    let basic_flow = FlowBuilder::new()
        .step(|ctx| async move {
            println!("  Step 1: Basic step executed");
            Ok(())
        })
        .named_step("step_2", |ctx| async move {
            println!("  Step 2: Named step executed");
            Ok(())
        })
        .step_if(
            |ctx| true, // Always execute
            |ctx| async move {
                println!("  Step 3: Conditional step executed");
                Ok(())
            },
        );

    let result = basic_flow.execute().await?;
    result.print_summary();

    // Macro example (if macros feature is enabled)
    #[cfg(feature = "macros")]
    {
        println!("2. Macro Example:");

        #[named_step("macro_step")]
        async fn my_macro_step(ctx: SharedContext) -> anyhow::Result<()> {
            println!("  Macro step executed with automatic logging");
            Ok(())
        }

        let macro_flow = FlowBuilder::new().step(|ctx| my_macro_step(ctx));

        let result = macro_flow.execute().await?;
        result.print_summary();
    }

    // Runtime features example (if runtime feature is enabled)
    #[cfg(feature = "runtime")]
    {
        use std::time::Duration;

        println!("3. Runtime Features Example:");

        // Scheduled execution
        let scheduled_flow = FlowBuilder::new()
            .step(|ctx| async move {
                println!("  Scheduled step executed");
                Ok(())
            })
            .schedule(ScheduleOptions::once_after(Duration::from_millis(100)));

        let result = scheduled_flow.execute().await?;
        result.print_summary();

        // Parallel execution
        let parallel_flow = FlowBuilder::new()
            .step(|ctx| async move {
                println!("  Parallel step 1");
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(())
            })
            .step(|ctx| async move {
                println!("  Parallel step 2");
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok(())
            })
            .execute_parallel();

        let result = parallel_flow.execute().await?;
        result.print_summary();
    }

    // Logger example (if logger feature is enabled)
    #[cfg(feature = "logger")]
    {
        println!("4. Logger Example:");

        let logger = Logger::new();
        logger.info("Flow execution completed");

        let context = FlowContext::default();
        logger.log_flow_summary(&context);
    }

    println!("=== Demo Completed ===");
    Ok(())
}
