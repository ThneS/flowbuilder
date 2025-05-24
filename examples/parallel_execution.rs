use flowbuilder::{Context, FlowBuilder, Result};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("FlowBuilder 并行执行示例");
    println!("===================");

    let mut ctx = Context::new();
    ctx.insert("results", Vec::<String>::new());

    // 定义一些异步任务
    async fn task1(ctx: &mut Context) -> Result<()> {
        println!("任务 1 开始");
        sleep(Duration::from_millis(100)).await;
        println!("任务 1 完成");
        if let Some(results) = ctx.get::<Vec<String>>("results") {
            let mut results = results.clone();
            results.push("任务 1 结果".to_string());
            ctx.insert("results", results);
        }
        Ok(())
    }

    async fn task2(ctx: &mut Context) -> Result<()> {
        println!("任务 2 开始");
        sleep(Duration::from_millis(150)).await;
        println!("任务 2 完成");
        if let Some(results) = ctx.get::<Vec<String>>("results") {
            let mut results = results.clone();
            results.push("任务 2 结果".to_string());
            ctx.insert("results", results);
        }
        Ok(())
    }

    async fn task3(ctx: &mut Context) -> Result<()> {
        println!("任务 3 开始");
        sleep(Duration::from_millis(200)).await;
        println!("任务 3 完成");
        if let Some(results) = ctx.get::<Vec<String>>("results") {
            let mut results = results.clone();
            results.push("任务 3 结果".to_string());
            ctx.insert("results", results);
        }
        Ok(())
    }

    // 使用并行执行
    FlowBuilder::new()
        .named_step("初始化", |ctx| {
            println!("初始化并行任务");
            Ok(())
        })
        .parallel_steps(vec![
            ("并行任务1", task1),
            ("并行任务2", task2),
            ("并行任务3", task3),
        ])
        .named_step("汇总结果", |ctx| {
            if let Some(results) = ctx.get::<Vec<String>>("results") {
                println!("\n所有任务完成，结果汇总：");
                for (i, result) in results.iter().enumerate() {
                    println!("{}. {}", i + 1, result);
                }
            }
            Ok(())
        })
        .run_all()
        .await?;

    Ok(())
}
