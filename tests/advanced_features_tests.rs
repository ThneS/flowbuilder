use anyhow::Result;
use flowbuilder::prelude::FlowBuilder;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_step_with_timeout_success() -> Result<()> {
    let flow = FlowBuilder::new().step_with_timeout(
        "quick_step",
        Duration::from_millis(500),
        |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("completed".to_string(), "true".to_string());
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        },
    );

    let result = flow.run_all_with_timeout(Duration::from_secs(1)).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_step_with_timeout_failure() -> Result<()> {
    let flow = FlowBuilder::new().step_with_timeout(
        "slow_step",
        Duration::from_millis(100),
        |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ok(())
        },
    );

    let result = flow.run_all_with_timeout(Duration::from_secs(1)).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timed out"));
    Ok(())
}

#[tokio::test]
async fn test_parallel_steps_with_join() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("setup".to_string(), "done".to_string());
            Ok(())
        })
        .parallel_steps_with_join(
            "test_parallel",
            vec![
                || {
                    FlowBuilder::new().named_step("task1", |_ctx| async move {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        Ok(())
                    })
                },
                || {
                    FlowBuilder::new().named_step("task2", |_ctx| async move {
                        tokio::time::sleep(Duration::from_millis(30)).await;
                        Ok(())
                    })
                },
            ],
        );

    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(
        guard.get_variable("test_parallel_parallel_success"),
        Some(&"2".to_string())
    );
    assert_eq!(
        guard.get_variable("test_parallel_parallel_failed"),
        Some(&"0".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_snapshot_and_rollback() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "10".to_string());
            guard.set_variable("status".to_string(), "active".to_string());
            Ok(())
        })
        .create_snapshot("checkpoint", "Initial state")
        .named_step("modify_state", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "20".to_string());
            guard.set_variable("new_var".to_string(), "temp".to_string());
            Ok(())
        })
        .rollback_to_snapshot("checkpoint")
        .named_step("verify_rollback", |ctx| async move {
            let guard = ctx.lock().await;
            assert_eq!(guard.get_variable("counter"), Some(&"10".to_string()));
            assert_eq!(guard.get_variable("new_var"), None);
            Ok(())
        });

    flow.run_all().await?;
    Ok(())
}

#[tokio::test]
async fn test_step_with_rollback_on_failure() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));

    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("important_data".to_string(), "original_value".to_string());
            Ok(())
        })
        .step_with_rollback("failing_operation", "safety_checkpoint", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("important_data".to_string(), "corrupted_value".to_string());
            guard.set_variable("temp_data".to_string(), "should_be_removed".to_string());
            drop(guard);
            anyhow::bail!("Operation failed")
        })
        .step_continue_on_error("verify_rollback", |ctx| async move {
            let guard = ctx.lock().await;
            // 数据应该被回滚到原始值
            assert_eq!(
                guard.get_variable("important_data"),
                Some(&"original_value".to_string())
            );
            // 临时数据应该被清除
            assert_eq!(guard.get_variable("temp_data"), None);
            Ok(())
        });

    // step_with_rollback 会在失败时返回错误，但会执行回滚
    // 我们用 step_continue_on_error 来验证回滚效果
    flow.run_all_with_context(ctx).await?;
    Ok(())
}

#[tokio::test]
async fn test_conditional_rollback() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("threshold".to_string(), "50".to_string());
            Ok(())
        })
        .step_with_conditional_rollback(
            "conditional_operation",
            "conditional_checkpoint",
            |ctx| async move {
                let mut guard = ctx.lock().await;
                guard.set_variable("test_value".to_string(), "100".to_string());
                Ok(())
            },
            |ctx| {
                // 如果 test_value > threshold 就回滚
                let test_value = ctx
                    .get_variable("test_value")
                    .unwrap_or(&"0".to_string())
                    .parse::<i32>()
                    .unwrap_or(0);
                let threshold = ctx
                    .get_variable("threshold")
                    .unwrap_or(&"50".to_string())
                    .parse::<i32>()
                    .unwrap_or(50);
                test_value > threshold
            },
        );

    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    // test_value 应该被回滚（因为 100 > 50）
    assert_eq!(guard.get_variable("test_value"), None);

    Ok(())
}

#[tokio::test]
async fn test_flow_level_timeout() -> Result<()> {
    let flow = FlowBuilder::new()
        .named_step("quick_step", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        })
        .named_step("another_quick_step", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        });

    // 流程应该在总超时时间内完成
    let result = flow.run_all_with_timeout(Duration::from_millis(200)).await;
    assert!(result.is_ok());

    // 测试超时情况
    let slow_flow = FlowBuilder::new().named_step("slow_step", |_ctx| async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    });

    let result = slow_flow
        .run_all_with_timeout(Duration::from_millis(100))
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timed out"));

    Ok(())
}
