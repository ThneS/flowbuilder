use anyhow::Result;

#[cfg(feature = "runtime")]
use {flowbuilder::prelude::*, std::sync::Arc, std::time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "runtime")]
    {
        println!("=== 高级并行执行演示 ===\n");

        // 创建共享上下文
        let _context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

        // 演示1：基本并行执行
        println!("1. 基本并行执行:");
        demo_basic_parallel().await?;

        // 演示2：并发限制
        println!("\n2. 并发限制演示:");
        demo_concurrency_limit().await?;

        // 演示3：超时处理
        println!("\n3. 超时处理演示:");
        demo_timeout_handling().await?;

        // 演示4：快速失败模式
        println!("\n4. 快速失败模式演示:");
        demo_fail_fast().await?;

        // 演示5：批处理执行
        println!("\n5. 批处理执行演示:");
        demo_batch_execution().await?;

        // 演示6：带监控的并行步骤
        println!("\n6. 并行步骤监控演示:");
        demo_parallel_step_monitoring().await?;

        println!("\n=== 演示完成 ===");
    }

    #[cfg(not(feature = "runtime"))]
    {
        println!("此示例需要启用 'runtime' 功能。");
        println!("请使用: cargo run --example advanced_parallel_demo --features runtime");
    }

    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_basic_parallel() -> Result<()> {
    let executor = ParallelExecutor::new();
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let steps = vec![
        ParallelStep::new("步骤1", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 步骤1");
            Ok(())
        }),
        ParallelStep::new("步骤2", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            println!("    执行 步骤2");
            Ok(())
        }),
        ParallelStep::new("步骤3", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            println!("    执行 步骤3");
            Ok(())
        }),
    ];

    let results = executor.execute_with_monitoring(steps, context).await?;
    println!(
        "  成功: {}, 失败: {}, 总时间: {:?}",
        results.success_count, results.failed_count, results.total_duration
    );
    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_concurrency_limit() -> Result<()> {
    let config = ParallelConfig::with_max_concurrency(2);
    let executor = ParallelExecutor::with_config(config);
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let steps = vec![
        ParallelStep::new("并发步骤1", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("    执行 并发步骤1");
            Ok(())
        }),
        ParallelStep::new("并发步骤2", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("    执行 并发步骤2");
            Ok(())
        }),
        ParallelStep::new("并发步骤3", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("    执行 并发步骤3");
            Ok(())
        }),
        ParallelStep::new("并发步骤4", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("    执行 并发步骤4");
            Ok(())
        }),
    ];

    let results = executor.execute_with_monitoring(steps, context).await?;
    println!(
        "  并发限制为2，成功: {}, 失败: {}, 总时间: {:?}",
        results.success_count, results.failed_count, results.total_duration
    );
    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_timeout_handling() -> Result<()> {
    let config = ParallelConfig::with_timeout(Duration::from_millis(250));
    let executor = ParallelExecutor::with_config(config);
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let steps = vec![
        ParallelStep::new("快速步骤", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 快速步骤");
            Ok(())
        }),
        ParallelStep::new("慢速步骤", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(500)).await; // 这个会超时
            println!("    执行 慢速步骤");
            Ok(())
        }),
        ParallelStep::new("正常步骤", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            println!("    执行 正常步骤");
            Ok(())
        }),
    ];

    let results = executor.execute_with_monitoring(steps, context).await?;
    println!(
        "  超时设置为250ms，成功: {}, 失败: {}, 错误数: {}",
        results.success_count,
        results.failed_count,
        results.errors.len()
    );

    for error in &results.errors {
        println!("    错误: {}", error);
    }
    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_fail_fast() -> Result<()> {
    let config = ParallelConfig::default().with_fail_fast();
    let executor = ParallelExecutor::with_config(config);
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let steps = vec![
        ParallelStep::new("正常步骤1", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 正常步骤1");
            Ok(())
        }),
        ParallelStep::new("失败步骤", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            Err(anyhow::anyhow!("失败步骤 故意失败"))
        }),
        ParallelStep::new("正常步骤2", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            println!("    执行 正常步骤2");
            Ok(())
        }),
    ];

    match executor.execute_with_monitoring(steps, context).await {
        Ok(results) => {
            println!(
                "  意外成功: {}, 失败: {}",
                results.success_count, results.failed_count
            );
        }
        Err(e) => {
            println!("  快速失败触发: {}", e);
        }
    }
    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_batch_execution() -> Result<()> {
    let config = ParallelConfig::with_max_concurrency(2); // Simulate batch of 2 concurrent at a time
    let executor = ParallelExecutor::with_config(config);
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let steps = vec![
        ParallelStep::new("批处理步骤1", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 批处理步骤1");
            Ok(())
        }),
        ParallelStep::new("批处理步骤2", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 批处理步骤2");
            Ok(())
        }),
        ParallelStep::new("批处理步骤3", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 批处理步骤3");
            Ok(())
        }),
        ParallelStep::new("批处理步骤4", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 批处理步骤4");
            Ok(())
        }),
        ParallelStep::new("批处理步骤5", |_ctx| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("    执行 批处理步骤5");
            Ok(())
        }),
    ];

    let results = executor.execute_with_monitoring(steps, context).await?;
    println!(
        "  批处理执行完成: 成功 {}, 失败 {}, 总时间: {:?}",
        results.success_count, results.failed_count, results.total_duration
    );
    Ok(())
}

#[cfg(feature = "runtime")]
async fn demo_parallel_step_monitoring() -> Result<()> {
    let config = ParallelConfig::with_max_concurrency(3).timeout(Duration::from_millis(400));
    let executor = ParallelExecutor::with_config(config);
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    let parallel_steps = vec![
        ParallelStep::new("数据处理步骤", |_ctx| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            println!("    数据处理完成");
            Ok(())
        }),
        ParallelStep::new("文件上传步骤", |_ctx| async {
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("    文件上传完成");
            Ok(())
        }),
        ParallelStep::new("邮件发送步骤", |_ctx| async {
            tokio::time::sleep(Duration::from_millis(250)).await;
            println!("    邮件发送完成");
            Ok(())
        }),
        ParallelStep::new("缓慢步骤", |_ctx| async {
            tokio::time::sleep(Duration::from_millis(500)).await; // 会超时
            println!("    缓慢步骤完成");
            Ok(())
        }),
    ];

    let results = executor
        .execute_with_monitoring(parallel_steps, context)
        .await?;
    println!(
        "  监控执行结果: 成功: {}, 失败: {}, 总时间: {:?}",
        results.success_count, results.failed_count, results.total_duration
    );

    for error in &results.errors {
        println!("    监控到的错误: {}", error);
    }
    Ok(())
}
