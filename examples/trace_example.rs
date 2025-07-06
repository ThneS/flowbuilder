use anyhow::Result;
use flowbuilder::prelude::FlowBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== FlowBuilder Advanced Features Example ===\n");

    // 示例 1: 超时控制
    println!("1. Timeout Control Example:");
    let timeout_flow = FlowBuilder::new()
        .named_step("quick_step", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("step1_done".to_string(), "true".to_string());
            println!("Quick step completed");
            Ok(())
        })
        .step_with_timeout("slow_step", Duration::from_millis(500), |_ctx| async move {
            println!("Starting slow operation...");
            tokio::time::sleep(Duration::from_millis(200)).await; // 不会超时
            println!("Slow operation completed within timeout");
            Ok(())
        })
        .step_with_timeout(
            "timeout_step",
            Duration::from_millis(100),
            |_ctx| async move {
                println!("This will timeout...");
                tokio::time::sleep(Duration::from_secs(1)).await; // 会超时
                println!("This should not print");
                Ok(())
            },
        );

    // 使用全流程超时运行
    match timeout_flow
        .run_all_with_timeout_and_trace_id(Duration::from_secs(2), "timeout-demo-123".to_string())
        .await
    {
        Ok(()) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed as expected: {}", e),
    }

    println!("\n{}\n", "=".repeat(50));

    // 示例 2: 并行执行与 Join
    println!("2. Parallel Execution with Join Example:");
    let parallel_flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("parallel_test".to_string(), "initialized".to_string());
            println!("Setup completed");
            Ok(())
        })
        .parallel_steps_with_join(
            "health_checks",
            vec![
                || {
                    FlowBuilder::new().named_step("database_check", |_ctx| async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        println!("Database health check: OK");
                        Ok(())
                    })
                },
                || {
                    FlowBuilder::new().named_step("api_check", |_ctx| async move {
                        tokio::time::sleep(Duration::from_millis(150)).await;
                        println!("API health check: OK");
                        Ok(())
                    })
                },
                || {
                    FlowBuilder::new().named_step("cache_check", |_ctx| async move {
                        tokio::time::sleep(Duration::from_millis(80)).await;
                        println!("Cache health check: OK");
                        Ok(())
                    })
                },
            ],
        )
        .named_step("verify_parallel_results", |ctx| async move {
            let guard = ctx.lock().await;
            let success = guard
                .get_variable("health_checks_parallel_success")
                .map(|s| s.as_str())
                .unwrap_or("0");
            let failed = guard
                .get_variable("health_checks_parallel_failed")
                .map(|s| s.as_str())
                .unwrap_or("0");
            println!(
                "Parallel health checks completed: {} success, {} failed",
                success, failed
            );
            Ok(())
        });

    parallel_flow
        .run_all_with_trace_id("parallel-demo-456".to_string())
        .await?;

    println!("\n{}\n", "=".repeat(50));

    // 示例 3: 快照与回滚
    println!("3. Snapshot and Rollback Example:");
    let snapshot_flow = FlowBuilder::new()
        .named_step("initial_setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "0".to_string());
            guard.set_variable("status".to_string(), "active".to_string());
            println!("Initial setup: counter=0, status=active");
            Ok(())
        })
        .create_snapshot("checkpoint_1", "After initial setup")
        .named_step("increment_counter", |ctx| async move {
            let mut guard = ctx.lock().await;
            let current = guard
                .get_variable("counter")
                .unwrap_or(&"0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            guard.set_variable("counter".to_string(), (current + 5).to_string());
            println!("Counter incremented to {}", current + 5);
            Ok(())
        })
        .step_with_rollback("risky_operation", "auto_rollback", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("temp_data".to_string(), "risky_value".to_string());
            println!("Performing risky operation...");
            drop(guard);

            // 模拟失败条件
            tokio::time::sleep(Duration::from_millis(100)).await;
            anyhow::bail!("Risky operation failed!")
        })
        .named_step("check_after_rollback", |ctx| async move {
            let guard = ctx.lock().await;
            let counter = guard
                .get_variable("counter")
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            let temp_data = guard.get_variable("temp_data");
            println!(
                "After rollback - Counter: {}, temp_data: {:?}",
                counter, temp_data
            );
            Ok(())
        })
        .step_with_conditional_rollback(
            "conditional_step",
            "conditional_checkpoint",
            |ctx| async move {
                let mut guard = ctx.lock().await;
                guard.set_variable("test_value".to_string(), "100".to_string());
                println!("Setting test_value to 100");
                Ok(())
            },
            |ctx| {
                // 如果 test_value 大于 50 就回滚
                let test_value = ctx
                    .get_variable("test_value")
                    .unwrap_or(&"0".to_string())
                    .parse::<i32>()
                    .unwrap_or(0);
                test_value > 50
            },
        )
        .named_step("final_check", |ctx| async move {
            let guard = ctx.lock().await;
            let test_value = guard.get_variable("test_value");
            println!("Final check - test_value: {:?}", test_value);
            Ok(())
        });

    snapshot_flow
        .run_all_with_trace_id("snapshot-demo-789".to_string())
        .await?;

    Ok(())
}
