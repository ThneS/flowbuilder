use anyhow::Result;
use flowbuilder::prelude::FlowBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== FlowBuilder 功能验证测试 ===\n");

    // 测试 1: 条件跳转 (Switch-Case)
    println!("1. 测试条件跳转 (Switch-Case):");
    let switch_flow = FlowBuilder::new()
        .named_step("设置用户类型", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("user_type".to_string(), "admin".to_string());
            guard.set_variable("action".to_string(), "delete".to_string());
            println!("✅ 设置: user_type=admin, action=delete");
            Ok(())
        })
        .step_switch_match_boxed("用户权限检查", |ctx| {
            let user_type = ctx.get_variable("user_type").cloned().unwrap_or_else(|| "unknown".to_string());
            match user_type.as_str() {
                "admin" => Some(Box::new(|| FlowBuilder::new()
                    .named_step("管理员权限", |ctx| async move {
                        let guard = ctx.lock().await;
                        let action = guard.get_variable("action").cloned().unwrap_or_else(|| "view".to_string());
                        println!("✅ 管理员用户执行操作: {}", action);
                        if action == "delete" {
                            println!("✅ 管理员拥有删除权限 - 操作允许");
                        }
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                "user" => Some(Box::new(|| FlowBuilder::new()
                    .named_step("普通用户权限", |_ctx| async move {
                        println!("✅ 普通用户 - 权限受限");
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
                _ => Some(Box::new(|| FlowBuilder::new()
                    .named_step("默认权限", |_ctx| async move {
                        println!("✅ 未知用户类型 - 使用默认权限");
                        Ok(())
                    })
                ) as Box<dyn Fn() -> FlowBuilder + Send>),
            }
        })
        .step_switch_match("操作匹配", |ctx| {
            let action = ctx.get_variable("action").cloned().unwrap_or_else(|| "view".to_string());
            match action.as_str() {
                "delete" => Some(|| FlowBuilder::new()
                    .named_step("执行删除", |_ctx| async move {
                        println!("✅ 执行删除操作，已通过权限验证");
                        Ok(())
                    })
                ),
                _ => None, // 其他情况跳过
            }
        });

    switch_flow
        .run_all_with_trace_id("switch-test-001".to_string())
        .await?;

    println!("\n{}\n", "=".repeat(50));

    // 测试 2: 全局错误处理器
    println!("2. 测试全局错误处理器:");
    let error_flow = FlowBuilder::new()
        .named_step("正常步骤", |ctx| async move {
            let mut guard = ctx.lock().await;
            guard.set_variable("step_count".to_string(), "1".to_string());
            println!("✅ 正常步骤执行成功");
            Ok(())
        })
        .named_step("失败步骤", |_ctx| async move {
            println!("❌ 这个步骤将失败...");
            anyhow::bail!("模拟的步骤失败");
        })
        .named_step("恢复步骤", |ctx| async move {
            let guard = ctx.lock().await;
            let last_error = guard.get_variable("last_error");
            let failed_step = guard.get_variable("failed_step");
            println!("✅ 恢复步骤执行，错误已处理");
            println!("   最后错误: {:?}", last_error);
            println!("   失败步骤: {:?}", failed_step);
            Ok(())
        })
        .with_global_error_handler_advanced(|step_name, ctx, error| {
            println!("🔧 全局错误处理器捕获到错误:");
            println!("   步骤: {}", step_name);
            println!("   错误: {}", error);
            
            // 记录错误到上下文
            ctx.set_variable("last_error".to_string(), error.to_string());
            ctx.set_variable("failed_step".to_string(), step_name.to_string());
            
            // 决定是否继续执行
            if step_name.contains("失败步骤") {
                println!("✅ 错误处理器: 从失败步骤恢复，继续工作流...");
                true // 继续执行
            } else {
                println!("❌ 错误处理器: 严重错误，停止工作流");
                false // 停止执行
            }
        });

    match error_flow
        .run_all_with_recovery("error-test-002".to_string())
        .await
    {
        Ok(()) => println!("✅ 流程通过错误恢复完成"),
        Err(e) => println!("❌ 流程失败: {}", e),
    }

    println!("\n{}\n", "=".repeat(50));

    // 测试 3: 超时控制
    println!("3. 测试超时控制:");
    let timeout_flow = FlowBuilder::new()
        .named_step("快速步骤", |_ctx| async move {
            println!("✅ 快速步骤完成");
            Ok(())
        })
        .step_with_timeout("超时步骤", Duration::from_millis(50), |_ctx| async move {
            println!("⏱️ 这个步骤会超时...");
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("❌ 这行不应该打印");
            Ok(())
        });

    match timeout_flow
        .run_all_with_timeout_and_trace_id(Duration::from_secs(1), "timeout-test-003".to_string())
        .await
    {
        Ok(()) => println!("✅ 超时流程意外完成"),
        Err(e) => println!("✅ 超时流程按预期失败: {}", e),
    }

    println!("\n{}\n", "=".repeat(50));

    // 测试 4: 并行执行
    println!("4. 测试并行执行:");
    let parallel_flow = FlowBuilder::new()
        .parallel_steps_with_join(
            "健康检查",
            vec![
                || FlowBuilder::new().named_step("数据库检查", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    println!("✅ 数据库健康检查: OK");
                    Ok(())
                }),
                || FlowBuilder::new().named_step("API检查", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    println!("✅ API健康检查: OK");
                    Ok(())
                }),
                || FlowBuilder::new().named_step("缓存检查", |_ctx| async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    println!("✅ 缓存健康检查: OK");
                    Ok(())
                }),
            ],
        )
        .named_step("验证结果", |ctx| async move {
            let guard = ctx.lock().await;
            let success = guard.get_variable("健康检查_parallel_success").cloned().unwrap_or_else(|| "0".to_string());
            let failed = guard.get_variable("健康检查_parallel_failed").cloned().unwrap_or_else(|| "0".to_string());
            println!("✅ 并行健康检查完成: {} 成功, {} 失败", success, failed);
            Ok(())
        });

    parallel_flow
        .run_all_with_trace_id("parallel-test-004".to_string())
        .await?;

    println!("\n=== 所有功能验证完成 ===");
    Ok(())
}
