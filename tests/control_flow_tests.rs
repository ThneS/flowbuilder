use flowbuilder::prelude::FlowBuilder;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_step_switch_str() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("mode".to_string(), "production".to_string());
            Ok(())
        })
        .step_switch_str(
            "mode_switch",
            "mode",
            vec![
                ("development", Box::new(|| FlowBuilder::new()
                    .named_step("dev_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "dev".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("production", Box::new(|| FlowBuilder::new()
                    .named_step("prod_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "prod".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
            ],
            Some(Box::new(|| FlowBuilder::new()
                .named_step("default_task", |ctx| async move {
                    let mut guard = ctx.lock().await;
                    guard.set_variable("executed".to_string(), "default".to_string());
                    Ok(())
                })
            ) as Box<dyn Fn() -> FlowBuilder + Send>),
        );

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("executed"), Some(&"prod".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_step_switch_str_default_case() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("mode".to_string(), "unknown".to_string());
            Ok(())
        })
        .step_switch_str(
            "mode_switch",
            "mode",
            vec![
                ("development", Box::new(|| FlowBuilder::new()
                    .named_step("dev_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("executed".to_string(), "dev".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
            ],
            Some(Box::new(|| FlowBuilder::new()
                .named_step("default_task", |ctx| async move {
                    let mut guard = ctx.lock().await;
                    guard.set_variable("executed".to_string(), "default".to_string());
                    Ok(())
                })
            ) as Box<dyn Fn() -> FlowBuilder + Send>),
        );

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("executed"), Some(&"default".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_step_switch_match() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("level".to_string(), "premium".to_string());
            guard.set_variable("balance".to_string(), "1500".to_string());
            Ok(())
        })
        .step_switch_match_boxed("access_control", |ctx| {
            let level = ctx.get_variable("level").map(|s| s.as_str()).unwrap_or("");
            let balance = ctx.get_variable("balance")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            match (level, balance) {
                ("premium", balance) if balance > 1000 => Some(Box::new(|| FlowBuilder::new()
                    .named_step("premium_access", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("access_level".to_string(), "premium_high".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("premium", _) => Some(Box::new(|| FlowBuilder::new()
                    .named_step("premium_limited", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("access_level".to_string(), "premium_low".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                _ => None,
            }
        });

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("access_level"), Some(&"premium_high".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_global_error_handler() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("step1", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("step1_done".to_string(), "true".to_string());
            Ok(())
        })
        .named_step("failing_step", |_ctx| async move {
            anyhow::bail!("Test error")
        })
        .named_step("step3", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("step3_done".to_string(), "true".to_string());
            Ok(())
        });

    // 使用全局错误处理器
    flow.run_all_with_context_and_recovery(ctx.clone(), |ctx, error| {
        // 记录错误但继续执行
        ctx.set_variable("error_handled".to_string(), "true".to_string());
        ctx.set_variable("error_message".to_string(), error.to_string());
        Ok(())
    }).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("step1_done"), Some(&"true".to_string()));
    assert_eq!(guard.get_variable("error_handled"), Some(&"true".to_string()));
    assert_eq!(guard.get_variable("step3_done"), Some(&"true".to_string()));
    assert!(guard.get_variable("error_message").unwrap().contains("Test error"));
    
    Ok(())
}

#[tokio::test]
async fn test_global_error_handler_wrapper() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("init", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("init_done".to_string(), "true".to_string());
            Ok(())
        })
        .named_step("critical_step", |_ctx| async move {
            anyhow::bail!("Critical failure")
        })
        .named_step("cleanup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("cleanup_done".to_string(), "true".to_string());
            Ok(())
        })
        .with_global_error_handler(|ctx, _error| {
            ctx.set_variable("wrapper_error_handled".to_string(), "true".to_string());
            ctx.set_variable("wrapper_error_type".to_string(), "critical".to_string());
            Ok(())
        });

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("init_done"), Some(&"true".to_string()));
    assert_eq!(guard.get_variable("wrapper_error_handled"), Some(&"true".to_string()));
    assert_eq!(guard.get_variable("cleanup_done"), Some(&"true".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_error_handler_failure() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("setup_done".to_string(), "true".to_string());
            Ok(())
        })
        .named_step("failing_step", |_ctx| async move {
            anyhow::bail!("Original error")
        });

    // 错误处理器本身也失败
    let result = flow.run_all_with_context_and_recovery(ctx.clone(), |_ctx, _error| {
        anyhow::bail!("Error handler also failed")
    }).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Error handler also failed"));
    
    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("setup_done"), Some(&"true".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_switch_with_no_match() -> Result<()> {
    let ctx = Arc::new(Mutex::new(flowbuilder::prelude::FlowContext::default()));
    
    let flow = FlowBuilder::new()
        .named_step("setup", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("status".to_string(), "unknown".to_string());
            Ok(())
        })
        .step_switch_str(
            "status_switch",
            "status",
            vec![
                ("active", Box::new(|| FlowBuilder::new()
                    .named_step("active_task", |ctx| async move {
                        let mut guard = ctx.lock().await;
                        guard.set_variable("result".to_string(), "active".to_string());
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
            ],
            None, // No default case
        )
        .named_step("final_step", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("final_done".to_string(), "true".to_string());
            Ok(())
        });

    flow.run_all_with_context(ctx.clone()).await?;

    let guard = ctx.lock().await;
    assert_eq!(guard.get_variable("result"), None); // No match, no execution
    assert_eq!(guard.get_variable("final_done"), Some(&"true".to_string())); // Flow continues
    
    Ok(())
}
