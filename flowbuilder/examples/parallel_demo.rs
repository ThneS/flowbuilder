#[cfg(feature = "runtime")]
use flowbuilder_runtime::FlowBuilderExt;

#[cfg(feature = "runtime")]
use flowbuilder_core::FlowBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Enhanced Parallel Execution Demo ===\n");

    #[cfg(feature = "runtime")]
    {
        use std::time::Duration;

        // Example 1: Basic parallel execution
        println!("1. Basic Parallel Execution:");
        let start = std::time::Instant::now();

        let _result = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Step 1 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(150)).await;
                println!("  Step 2 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(80)).await;
                println!("  Step 3 completed");
                Ok(())
            })
            .execute_parallel()
            .execute()
            .await?;

        println!("  Parallel execution took: {:?}", start.elapsed());
        println!("  (Should be ~150ms instead of ~330ms sequential)\n");

        // Example 2: Limited concurrency
        println!("2. Limited Concurrency (max 2 concurrent):");
        let start = std::time::Instant::now();

        let _result = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Concurrent step 1 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Concurrent step 2 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Concurrent step 3 completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Concurrent step 4 completed");
                Ok(())
            })
            .execute_parallel()
            .max_concurrency(2)
            .execute()
            .await?;

        println!("  Limited concurrency took: {:?}", start.elapsed());
        println!("  (Should be ~200ms with max 2 concurrent)\n");

        // Example 3: Detailed results
        println!("3. Detailed Execution Results:");
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

        println!("  Execution Results:");
        println!("    - Success count: {}", results.success_count);
        println!("    - Failed count: {}", results.failed_count);
        println!("    - Total duration: {:?}", results.total_duration);
        println!("    - Errors: {}", results.errors.len());
        println!();

        // Example 4: Error handling with fail-fast
        println!("4. Error Handling (fail-fast disabled):");
        let results = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Step 1 succeeded");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Err(anyhow::anyhow!("Step 2 failed intentionally"))
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(75)).await;
                println!("  Step 3 succeeded");
                Ok(())
            })
            .execute_parallel()
            .execute_detailed()
            .await?;

        println!("  Results with partial failure:");
        println!("    - Success count: {}", results.success_count);
        println!("    - Failed count: {}", results.failed_count);
        println!(
            "    - Errors: {:?}",
            results
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );
        println!();

        // Example 5: Timeout configuration
        println!("5. Step Timeout Configuration:");
        let results = FlowBuilder::new()
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                println!("  Fast step completed");
                Ok(())
            })
            .step(|_ctx| async move {
                tokio::time::sleep(Duration::from_millis(200)).await;
                println!("  This should timeout");
                Ok(())
            })
            .execute_parallel()
            .timeout(Duration::from_millis(100))
            .execute_detailed()
            .await?;

        println!("  Results with timeout:");
        println!("    - Success count: {}", results.success_count);
        println!("    - Failed count: {}", results.failed_count);
        println!(
            "    - Errors: {:?}",
            results
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );
        println!();

        // Example 6: Batched execution
        println!("6. Batched Execution:");
        let results = FlowBuilder::new()
            .step(|_ctx| async move {
                println!("  Batch step 1");
                Ok(())
            })
            .step(|_ctx| async move {
                println!("  Batch step 2");
                Ok(())
            })
            .step(|_ctx| async move {
                println!("  Batch step 3");
                Ok(())
            })
            .step(|_ctx| async move {
                println!("  Batch step 4");
                Ok(())
            })
            .execute_parallel()
            .batch_size(2)
            .execute_detailed()
            .await?;

        println!("  Batched execution results:");
        println!("    - Success count: {}", results.success_count);
        println!("    - Total duration: {:?}", results.total_duration);
    }

    #[cfg(not(feature = "runtime"))]
    {
        println!("Runtime features not enabled. Enable with --features runtime");
    }

    println!("=== Demo Completed ===");
    Ok(())
}
