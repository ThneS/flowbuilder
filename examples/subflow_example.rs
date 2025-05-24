use flowbuilder::{Context, FlowBuilder, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("FlowBuilder 子流程示例");
    println!("===================");

    let mut ctx = Context::new();
    ctx.insert("total_steps", 0);

    // 定义子流程
    async fn data_processing_subflow(ctx: Context) -> Result<Context> {
        println!("\n开始数据处理子流程");
        let mut ctx = ctx;

        FlowBuilder::new()
            .named_step("数据验证", |ctx| {
                println!("验证数据...");
                ctx.insert("is_valid", true);
                Ok(())
            })
            .step_if(
                |ctx| ctx.get::<bool>("is_valid").unwrap_or(false),
                |ctx| {
                    println!("处理有效数据...");
                    ctx.insert("processed", true);
                    Ok(())
                },
            )
            .step_handle_error(
                "错误恢复",
                |ctx| {
                    if !ctx.get::<bool>("is_valid").unwrap_or(false) {
                        Err("数据无效".into())
                    } else {
                        Ok(())
                    }
                },
                |ctx, e| {
                    println!("处理错误: {}", e);
                    ctx.insert("error_handled", true);
                    Ok(())
                },
            )
            .run_all()
            .await?;

        Ok(ctx)
    }

    // 主流程
    FlowBuilder::new()
        .named_step("初始化", |ctx| {
            println!("初始化主流程");
            ctx.insert("batch_id", "BATCH-001");
            Ok(())
        })
        // 添加子流程
        .subflow("数据处理", data_processing_subflow)
        // 条件子流程
        .subflow_if(
            |ctx| ctx.get::<bool>("processed").unwrap_or(false),
            |ctx| async move {
                println!("\n开始后处理子流程");
                FlowBuilder::new()
                    .named_step("清理", |ctx| {
                        println!("清理临时数据...");
                        ctx.remove::<bool>("is_valid");
                        Ok(())
                    })
                    .named_step("汇总", |ctx| {
                        let total = ctx.get::<i32>("total_steps").unwrap_or(0);
                        println!("总步骤数: {}", total);
                        ctx.insert("total_steps", total + 1);
                        Ok(())
                    })
                    .run_all()
                    .await?;
                Ok(ctx)
            },
        )
        // 等待子流程完成
        .wait_until(
            |ctx| ctx.get::<i32>("total_steps").unwrap_or(0) > 0,
            Duration::from_millis(100),
            5,
        )
        .run_all()
        .await?;

    // 打印最终结果
    println!("\n最终结果:");
    println!(
        "批次 ID: {}",
        ctx.get::<&str>("batch_id").unwrap_or("未设置")
    );
    println!(
        "处理状态: {}",
        if ctx.get::<bool>("processed").unwrap_or(false) {
            "成功"
        } else {
            "失败"
        }
    );
    println!("总步骤数: {}", ctx.get::<i32>("total_steps").unwrap_or(0));

    Ok(())
}
