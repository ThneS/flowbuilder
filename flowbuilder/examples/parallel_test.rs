#[cfg(feature = "runtime")]
use flowbuilder_runtime::FlowBuilderExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Parallel Execution Test ===\n");

    #[cfg(feature = "runtime")]
    {
        use std::time::Duration;

        // Test 1: Basic parallel execution
        println!("1. Basic Parallel Execution Test:");
        let start = std::time::Instant::now();

        let _result = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Step 1 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Step 2 completed");
                Ok(())
            })
            .execute_parallel()
            .execute()
            .await?;

        let elapsed = start.elapsed();
        println!("  Execution time: {:?}", elapsed);
        println!("  (Should be ~100ms, not ~150ms sequential)\n");

        // Test 2: Detailed results
        println!("2. Detailed Results Test:");
        let results = FlowBuilder::new()
            .step(|_ctx| async move {
                println!("  Success step executed");
                Ok(())
            })
            .step(|_ctx| async move {
                println!("  Another success step executed");
                Ok(())
            })
            .execute_parallel()
            .execute_detailed()
            .await?;

        println!("  Results:");
        println!("    - Success count: {}", results.success_count);
        println!("    - Failed count: {}", results.failed_count);
        println!("    - Total duration: {:?}", results.total_duration);
        println!();

        // Test 3: Concurrency limit
        println!("3. Concurrency Limit Test:");
        let start = std::time::Instant::now();

        let _result = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Limited step 1 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Limited step 2 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Limited step 3 completed");
                Ok(())
            })
            .execute_parallel()
            .max_concurrency(2)
            .execute()
            .await?;

        let elapsed = start.elapsed();
        println!("  Execution time with max_concurrency(2): {:?}", elapsed);
        println!("  (Should be ~100ms with 2 concurrent)\n");
    }

    #[cfg(not(feature = "runtime"))]
    {
        println!("Runtime features not enabled. Enable with --features runtime");
    }

    println!("=== Test Completed ===");
    Ok(())
}
