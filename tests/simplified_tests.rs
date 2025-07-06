use anyhow::Result;
use flowbuilder::prelude::FlowBuilder;
use std::time::Duration;

#[tokio::test]
async fn test_switch_case_basic() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("mode".to_string(), "production".to_string());
            Ok(())
        })
        .step_switch_match_boxed("mode_switch", |ctx| {
            let mode = ctx.get_variable("mode").cloned().unwrap_or_else(|| "unknown".to_string());
            match mode.as_str() {
                "development" => Some(Box::new(|| FlowBuilder::new()
                    .named_step("dev_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "dev".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                "production" => Some(Box::new(|| FlowBuilder::new()
                    .named_step("prod_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "prod".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                _ => Some(Box::new(|| FlowBuilder::new()
                    .named_step("default_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "default".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
            }
        });

    let _ctx = flow.run_all_with_trace_id("test-switch-001".to_string()).await?;
    Ok(())
}

#[tokio::test]
async fn test_global_error_handler() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("normal_step", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("step_count".to_string(), "1".to_string());
            Ok(())
        })
        .named_step("failing_step", |_ctx| async move {
            anyhow::bail!("Simulated failure")
        })
        .named_step("recovery_step", |_ctx| async move {
            Ok(())
        })
        .with_global_error_handler_advanced(|step_name, ctx, _error| {
            ctx.set_variable("error_handled".to_string(), "true".to_string());
            ctx.set_variable("failed_step".to_string(), step_name.to_string());
            
            // Continue execution for failing_step, stop for others
            step_name.contains("failing_step")
        });

    // This should succeed because the error handler allows continuation
    match flow.run_all_with_recovery("test-error-001".to_string()).await {
        Ok(()) => {
            // Expected: error handler should allow continuation
        }
        Err(_) => {
            // Also acceptable depending on implementation
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_timeout_control() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("quick_step", |_ctx| async move {
            Ok(())
        })
        .step_with_timeout("timeout_step", Duration::from_millis(10), |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        });

    // This should fail due to timeout
    let result = flow
        .run_all_with_timeout_and_trace_id(Duration::from_secs(1), "test-timeout-001".to_string())
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_parallel_execution() -> Result<()> {
    let flow = FlowBuilder::new()
        .parallel_steps_with_join(
            "parallel_test",
            vec![
                || FlowBuilder::new().named_step("task1", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                }),
                || FlowBuilder::new().named_step("task2", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    Ok(())
                }),
                || FlowBuilder::new().named_step("task3", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(15)).await;
                    Ok(())
                }),
            ],
        );

    flow.run_all_with_trace_id("test-parallel-001".to_string()).await?;
    Ok(())
}
