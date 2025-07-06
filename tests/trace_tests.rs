use anyhow::Result;
use flowbuilder::prelude::{FlowBuilder, FlowContext};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_trace_id_functionality() -> Result<()> {
    let custom_trace_id = "test-trace-123".to_string();
    let ctx = Arc::new(Mutex::new(FlowContext::new_with_trace_id(
        custom_trace_id.clone(),
    )));

    let flow = FlowBuilder::new().named_step("test_step", |ctx| async move {
        let mut guard = ctx.lock().await;
        guard.set_variable("test_var".to_string(), "test_value".to_string());
        Ok(())
    });

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.trace_id, custom_trace_id);
    assert_eq!(
        guard.get_variable("test_var"),
        Some(&"test_value".to_string())
    );
    assert!(!guard.step_logs.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_step_continue_on_error() -> Result<()> {
    let ctx = Arc::new(Mutex::new(FlowContext::default()));

    let flow = FlowBuilder::new()
        .step_continue_on_error("failing_step", |_ctx| async move {
            anyhow::bail!("This should not stop the flow")
        })
        .named_step("success_step", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("after_error".to_string(), "executed".to_string());
            Ok(())
        });

    // 流程应该成功完成，即使有一个步骤失败
    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(
        guard.get_variable("after_error"),
        Some(&"executed".to_string())
    );
    assert!(!guard.ok); // ok 应该是 false 因为有步骤失败了
    assert!(!guard.errors.is_empty()); // 应该有错误记录

    Ok(())
}

#[tokio::test]
async fn test_step_wait_until() -> Result<()> {
    let ctx = Arc::new(Mutex::new(FlowContext::default()));

    // 设置一个变量，然后等待它变为特定值
    {
        let mut guard = ctx.lock().await;
        guard.set_variable("counter".to_string(), "0".to_string());
    }

    let flow = FlowBuilder::new()
        .named_step("increment", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "5".to_string());
            Ok(())
        })
        .step_wait_until(
            "wait_for_counter",
            |ctx| {
                let counter = ctx
                    .get_variable("counter")
                    .unwrap_or(&"0".to_string())
                    .parse::<i32>()
                    .unwrap_or(0);
                counter >= 5
            },
            Duration::from_millis(10),
            10,
        )
        .named_step("after_wait", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("completed".to_string(), "true".to_string());
            Ok(())
        });

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("completed"), Some(&"true".to_string()));

    Ok(())
}
