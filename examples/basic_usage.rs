use flowbuilder::{Context, FlowBuilder, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("FlowBuilder 基本用法示例");
    println!("=====================");

    // 创建一个新的流程
    let mut ctx = Context::new();
    ctx.insert("counter", 0);

    FlowBuilder::new()
        // 添加一个命名步骤
        .named_step("初始化", |ctx| {
            println!("执行初始化步骤");
            ctx.insert("message", "Hello, FlowBuilder!");
            Ok(())
        })
        // 添加条件步骤
        .step_if(
            |ctx| ctx.get::<i32>("counter").unwrap_or(0) < 5,
            |ctx| {
                let counter = ctx.get::<i32>("counter").unwrap_or(0);
                println!("计数器: {}", counter);
                ctx.insert("counter", counter + 1);
                Ok(())
            },
        )
        // 添加等待逻辑
        .wait_until(
            |ctx| ctx.get::<i32>("counter").unwrap_or(0) >= 5,
            Duration::from_millis(100),
            10,
        )
        // 添加错误处理
        .step_handle_error(
            "错误处理",
            |ctx| {
                println!("尝试执行可能失败的操作");
                if ctx.get::<i32>("counter").unwrap_or(0) > 3 {
                    Err("计数器太大".into())
                } else {
                    Ok(())
                }
            },
            |ctx, e| {
                println!("捕获到错误: {}", e);
                Ok(())
            },
        )
        // 运行所有步骤
        .run_all()
        .await?;

    // 打印最终结果
    println!("\n最终结果:");
    println!("消息: {}", ctx.get::<&str>("message").unwrap_or("未设置"));
    println!("计数器: {}", ctx.get::<i32>("counter").unwrap_or(0));

    Ok(())
}
