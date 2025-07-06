use anyhow::Result;
use flowbuilder::prelude::FlowBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Control Flow Examples ===\n");

    // 示例 1: Switch-Case 多路分支（字符串匹配）
    println!("1. String-based Switch-Case Example:");
    let switch_flow = FlowBuilder::new()
        .named_step("setup_mode", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("environment".to_string(), "production".to_string());
            guard.set_variable("feature_flag".to_string(), "enabled".to_string());
            println!("Environment set to production");
            Ok(())
        })
        .step_switch_str(
            "environment_switch",
            "environment",
            vec![
                ("development", Box::new(|| {
                    FlowBuilder::new()
                        .named_step("dev_setup", |_ctx| async move {
                            println!("Setting up development environment:");
                            println!("- Debug mode enabled");
                            println!("- Hot reload activated");
                            println!("- Mock services connected");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("staging", Box::new(|| {
                    FlowBuilder::new()
                        .named_step("staging_setup", |_ctx| async move {
                            println!("Setting up staging environment:");
                            println!("- Performance monitoring enabled");
                            println!("- Test data loaded");
                            println!("- Load balancer configured");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("production", Box::new(|| {
                    FlowBuilder::new()
                        .named_step("prod_setup", |_ctx| async move {
                            println!("Setting up production environment:");
                            println!("- Security hardening applied");
                            println!("- Monitoring alerts configured");
                            println!("- Database connections pooled");
                            Ok(())
                        })
                        .named_step("health_check", |_ctx| async move {
                            println!("Running production health checks...");
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            println!("All systems operational");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
            ],
            Some(Box::new(|| {
                FlowBuilder::new().named_step("default_setup", |_ctx| async move {
                    println!("Unknown environment, using default configuration");
                    Ok(())
                })
            }) as Box<dyn Fn() -> FlowBuilder + Send>),
        )
        .step_switch_str(
            "feature_switch",
            "feature_flag",
            vec![
                ("enabled", Box::new(|| {
                    FlowBuilder::new().named_step("enable_features", |_ctx| async move {
                        println!("Advanced features enabled");
                        Ok(())
                    })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("disabled", Box::new(|| {
                    FlowBuilder::new().named_step("basic_mode", |_ctx| async move {
                        println!("Running in basic mode");
                        Ok(())
                    })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
            ],
            None,
        );

    switch_flow
        .run_all_with_trace_id("switch-demo-123".to_string())
        .await?;

    println!("\n{}\n", "=".repeat(60));

    // 示例 2: 高级匹配器 Switch-Case
    println!("2. Advanced Matcher Switch-Case Example:");
    let advanced_switch_flow = FlowBuilder::new()
        .named_step("setup_data", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("user_level".to_string(), "premium".to_string());
            guard.set_variable("account_balance".to_string(), "1500".to_string());
            guard.set_variable("region".to_string(), "us-west".to_string());
            println!("User data initialized");
            Ok(())
        })
        .step_switch_match_boxed("user_access_control", |ctx| {
            let user_level = ctx.get_variable("user_level").map(|s| s.as_str()).unwrap_or("");
            let balance = ctx
                .get_variable("account_balance")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let region = ctx.get_variable("region").map(|s| s.as_str()).unwrap_or("");

            // 复杂的条件匹配逻辑
            match (user_level, balance, region) {
                ("premium", balance, _) if balance > 1000 => Some(Box::new(|| {
                    FlowBuilder::new()
                        .named_step("premium_access", |_ctx| async move {
                            println!("Granting premium access with high balance");
                            println!("- Unlimited API calls");
                            println!("- Priority support");
                            println!("- Advanced analytics");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("premium", _, _) => Some(Box::new(|| {
                    FlowBuilder::new()
                        .named_step("premium_limited", |_ctx| async move {
                            println!("Premium access with limited features (low balance)");
                            println!("- Standard API limits");
                            println!("- Premium support");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("standard", _, region) if region.starts_with("us-") => Some(Box::new(|| {
                    FlowBuilder::new()
                        .named_step("us_standard", |_ctx| async move {
                            println!("US standard access");
                            println!("- Regional compliance applied");
                            println!("- US data centers only");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                ("standard", _, _) => Some(Box::new(|| {
                    FlowBuilder::new()
                        .named_step("global_standard", |_ctx| async move {
                            println!("Standard access (global)");
                            println!("- Basic features enabled");
                            Ok(())
                        })
                }) as Box<dyn Fn() -> FlowBuilder + Send>),
                _ => None, // No matching case
            }
        });

    advanced_switch_flow
        .run_all_with_trace_id("advanced-switch-456".to_string())
        .await?;

    println!("\n{}\n", "=".repeat(60));

    // 示例 3: 全局错误处理器
    println!("3. Global Error Handler Example:");
    let error_flow = FlowBuilder::new()
        .named_step("step1", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("counter".to_string(), "0".to_string());
            println!("Step 1: Initial setup completed");
            Ok(())
        })
        .named_step("step2", |ctx| async move {
            let mut guard = ctx.lock().await;
            let counter = guard
                .get_variable("counter")
                .unwrap_or(&"0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            guard.set_variable("counter".to_string(), (counter + 10).to_string());
            println!("Step 2: Counter incremented to {}", counter + 10);
            Ok(())
        })
        .named_step("failing_step", |_ctx| async move {
            println!("Step 3: This step will fail...");
            anyhow::bail!("Simulated network error")
        })
        .named_step("step4", |ctx| async move {
            let guard = ctx.lock().await;
            let counter = guard
                .get_variable("counter")
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            println!("Step 4: Final counter value: {}", counter);
            Ok(())
        })
        .named_step("cleanup", |_ctx| async move {
            println!("Step 5: Cleanup completed");
            Ok(())
        });

    // 使用全局错误处理器运行
    error_flow
        .run_all_with_recovery_and_trace_id(
            |ctx, error| {
                println!("🚨 Global Error Handler activated!");
                println!("   Error: {}", error);

                // 记录错误但继续执行
                ctx.set_variable("error_occurred".to_string(), "true".to_string());
                ctx.set_variable("error_message".to_string(), error.to_string());
                ctx.set_variable("recovery_action".to_string(), "logged_and_continued".to_string());

                println!("   ✅ Error logged, flow will continue");
                println!("   📊 Error metrics updated");
                println!("   🔄 Recovery action: continue execution");

                Ok(()) // 返回 Ok 表示错误已处理，流程继续
            },
            "error-handling-demo-789".to_string(),
        )
        .await?;

    println!("\n{}\n", "=".repeat(60));

    // 示例 4: 使用 FlowBuilderWithErrorHandler 包装器
    println!("4. FlowBuilder with Error Handler Wrapper Example:");
    let wrapper_flow = FlowBuilder::new()
        .named_step("init", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("process_id".to_string(), "proc_001".to_string());
            println!("Process initialized: proc_001");
            Ok(())
        })
        .named_step("critical_operation", |_ctx| async move {
            println!("Attempting critical operation...");
            anyhow::bail!("Critical system failure!")
        })
        .named_step("finalize", |_ctx| async move {
            println!("Process finalized successfully");
            Ok(())
        })
        .with_global_error_handler(|ctx, error| {
            println!("🛡️  Critical Error Handler:");
            println!("   Process ID: {}", ctx.get_variable("process_id").unwrap_or(&"unknown".to_string()));
            println!("   Error: {}", error);
            
            // 设置恢复策略
            ctx.set_variable("error_handled_by".to_string(), "critical_handler".to_string());
            ctx.set_variable("recovery_timestamp".to_string(), "2025-01-06T12:00:00Z".to_string());
            
            println!("   🔧 Applying emergency recovery protocol");
            println!("   ✅ System stabilized, continuing with degraded mode");
            
            Ok(())
        });

    wrapper_flow
        .run_all_with_trace_id("wrapper-demo-999".to_string())
        .await?;

    println!("\n=== All Examples Completed Successfully! ===");
    Ok(())
}
