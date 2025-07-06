//! # 调度器与编排器集成演示
//!
//! 演示 TaskScheduler 和 FlowOrchestrator 如何协同工作，
//! 实现复杂的工作流调度和编排功能

use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_core::{Flow, FlowBuilder};
use flowbuilder_runtime::orchestrator::{
    BranchCondition, ErrorRecoveryStrategy, FlowNode, FlowOrchestrator, OrchestratorConfig,
    RetryConfig,
};
use flowbuilder_runtime::scheduler::{
    Priority, ScheduledTask, SchedulerConfig, SchedulingStrategy, TaskScheduler, TaskStatus,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

/// 演示基本的调度器功能
async fn demo_basic_scheduler() -> Result<()> {
    println!("=== 基本调度器演示 ===");

    let config = SchedulerConfig {
        max_concurrent_tasks: 3,
        strategy: SchedulingStrategy::Priority,
        task_timeout: Some(Duration::from_secs(10)),
        max_retries: 2,
        retry_delay: Duration::from_millis(500),
        enable_dependency_check: true,
    };

    let scheduler = TaskScheduler::new(config);

    // 创建几个不同优先级的任务
    let tasks = vec![
        ("数据清理任务", Priority::Low),
        ("报告生成任务", Priority::Normal),
        ("系统监控任务", Priority::High),
        ("安全检查任务", Priority::Critical),
    ];

    let mut task_ids = Vec::new();

    for (name, priority) in tasks {
        let task_name = name.to_string();
        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: task_name.clone(),
            description: Some(format!("演示任务: {}", task_name)),
            priority,
            estimated_duration: Some(Duration::from_millis(500)),
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            task_fn: Box::new(move |_ctx| {
                let name = task_name.clone();
                tokio::spawn(async move {
                    println!("  执行任务: {}", name);
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    println!("  任务完成: {}", name);
                    Ok(())
                })
            }),
            metadata: HashMap::new(),
        };

        let task_id = scheduler.submit_task(task).await?;
        task_ids.push(task_id);
    }

    // 模拟调度器运行
    println!("启动调度器...");
    tokio::spawn(async move {
        let _ = scheduler.start().await;
    });

    // 等待任务完成
    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("调度器演示完成\n");
    Ok(())
}

/// 演示编排器的复杂流程编排
async fn demo_flow_orchestrator() -> Result<()> {
    println!("=== 流程编排器演示 ===");

    let config = OrchestratorConfig {
        max_parallelism: 5,
        global_timeout: Some(Duration::from_secs(30)),
        enable_checkpoints: true,
        checkpoint_interval: Duration::from_secs(5),
        verbose_logging: true,
    };

    let mut orchestrator = FlowOrchestrator::with_config(config);

    // 创建示例流程
    let data_input_flow = FlowBuilder::new()
        .name("数据输入流程")
        .step("read_data", |_ctx| {
            println!("  读取输入数据...");
            Ok(())
        })
        .step("validate_data", |_ctx| {
            println!("  验证数据格式...");
            Ok(())
        })
        .build();

    let data_processing_flow = FlowBuilder::new()
        .name("数据处理流程")
        .step("transform", |_ctx| {
            println!("  转换数据格式...");
            Ok(())
        })
        .step("calculate", |_ctx| {
            println!("  执行计算...");
            Ok(())
        })
        .build();

    let data_output_flow = FlowBuilder::new()
        .name("数据输出流程")
        .step("format_output", |_ctx| {
            println!("  格式化输出...");
            Ok(())
        })
        .step("save_results", |_ctx| {
            println!("  保存结果...");
            Ok(())
        })
        .build();

    // 定义流程节点
    let input_node = FlowNode {
        id: "input".to_string(),
        name: "数据输入节点".to_string(),
        description: Some("负责数据输入和验证".to_string()),
        flow: Some(data_input_flow),
        condition: Some(BranchCondition::Boolean(true)),
        next_nodes: vec!["processing".to_string()],
        error_recovery: ErrorRecoveryStrategy::Retry {
            max_attempts: 3,
            delay: Duration::from_secs(1),
        },
        timeout: Some(Duration::from_secs(10)),
        retry_config: Some(RetryConfig::default()),
    };

    let processing_node = FlowNode {
        id: "processing".to_string(),
        name: "数据处理节点".to_string(),
        description: Some("负责数据处理和计算".to_string()),
        flow: Some(data_processing_flow),
        condition: None,
        next_nodes: vec!["output".to_string()],
        error_recovery: ErrorRecoveryStrategy::Fallback {
            fallback_flow_id: "backup_processing".to_string(),
        },
        timeout: Some(Duration::from_secs(15)),
        retry_config: Some(RetryConfig {
            max_attempts: 2,
            delay: Duration::from_secs(2),
            backoff_multiplier: 1.5,
            max_delay: Duration::from_secs(10),
        }),
    };

    let output_node = FlowNode {
        id: "output".to_string(),
        name: "数据输出节点".to_string(),
        description: Some("负责结果输出和保存".to_string()),
        flow: Some(data_output_flow),
        condition: None,
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::Skip,
        timeout: Some(Duration::from_secs(5)),
        retry_config: None,
    };

    // 创建备用处理节点
    let backup_processing_flow = FlowBuilder::new()
        .name("备用处理流程")
        .step("simple_process", |_ctx| {
            println!("  执行简化处理...");
            Ok(())
        })
        .build();

    let backup_processing_node = FlowNode {
        id: "backup_processing".to_string(),
        name: "备用处理节点".to_string(),
        description: Some("当主处理节点失败时的备用方案".to_string()),
        flow: Some(backup_processing_flow),
        condition: None,
        next_nodes: vec!["output".to_string()],
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: Some(Duration::from_secs(8)),
        retry_config: None,
    };

    // 添加节点到编排器
    orchestrator
        .add_node(input_node)
        .add_node(processing_node)
        .add_node(output_node)
        .add_node(backup_processing_node);

    // 设置依赖关系
    orchestrator
        .add_dependency("processing".to_string(), "input".to_string())
        .add_dependency("output".to_string(), "processing".to_string());

    // 创建检查点
    orchestrator.create_checkpoint("start").await?;

    println!("开始执行工作流编排...");
    let results = orchestrator.execute_all().await?;

    println!("编排结果:");
    for (node_id, _context) in results {
        println!("  节点 {} 已完成", node_id);
    }

    // 显示统计信息
    let stats = orchestrator.get_stats().await;
    println!("编排统计:");
    println!("  成功节点: {}", stats.successful_nodes);
    println!("  失败节点: {}", stats.failed_nodes);
    println!("  总耗时: {:?}", stats.total_duration);

    println!("流程编排器演示完成\n");
    Ok(())
}

/// 演示调度器与编排器的集成使用
async fn demo_integrated_workflow() -> Result<()> {
    println!("=== 集成工作流演示 ===");

    // 创建调度器
    let scheduler_config = SchedulerConfig {
        max_concurrent_tasks: 2,
        strategy: SchedulingStrategy::Priority,
        task_timeout: Some(Duration::from_secs(30)),
        max_retries: 3,
        retry_delay: Duration::from_secs(1),
        enable_dependency_check: true,
    };
    let scheduler = Arc::new(TaskScheduler::new(scheduler_config));

    // 创建编排器
    let orchestrator_config = OrchestratorConfig {
        max_parallelism: 3,
        global_timeout: Some(Duration::from_secs(60)),
        enable_checkpoints: true,
        checkpoint_interval: Duration::from_secs(10),
        verbose_logging: true,
    };
    let orchestrator = Arc::new(Mutex::new(FlowOrchestrator::with_config(orchestrator_config)));

    // 创建多个工作流，每个作为一个调度任务
    let workflows = vec![
        ("用户注册工作流", Priority::High),
        ("数据备份工作流", Priority::Normal),
        ("日志清理工作流", Priority::Low),
    ];

    let mut task_ids = Vec::new();

    for (workflow_name, priority) in workflows {
        let name = workflow_name.to_string();
        let orchestrator_clone = orchestrator.clone();

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: name.clone(),
            description: Some(format!("执行工作流: {}", name)),
            priority,
            estimated_duration: Some(Duration::from_secs(5)),
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            task_fn: Box::new(move |_ctx| {
                let name = name.clone();
                let orchestrator = orchestrator_clone.clone();
                tokio::spawn(async move {
                    println!("  开始执行工作流: {}", name);

                    // 创建简单的工作流
                    let flow = FlowBuilder::new()
                        .name(&name)
                        .step("init", move |_ctx| {
                            println!("    {} - 初始化", name);
                            Ok(())
                        })
                        .step("process", move |_ctx| {
                            println!("    {} - 处理", name);
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            Ok(())
                        })
                        .step("finalize", move |_ctx| {
                            println!("    {} - 完成", name);
                            Ok(())
                        })
                        .build();

                    // 通过编排器执行工作流
                    let node = FlowNode {
                        id: format!("{}_node", name.replace(" ", "_")),
                        name: name.clone(),
                        description: Some(format!("执行 {} 的节点", name)),
                        flow: Some(flow),
                        condition: None,
                        next_nodes: Vec::new(),
                        error_recovery: ErrorRecoveryStrategy::Retry {
                            max_attempts: 2,
                            delay: Duration::from_millis(500),
                        },
                        timeout: Some(Duration::from_secs(10)),
                        retry_config: None,
                    };

                    {
                        let mut orch = orchestrator.lock().await;
                        orch.add_node(node);
                        let _results = orch.execute_all().await?;
                    }

                    println!("  工作流完成: {}", name);
                    Ok(())
                })
            }),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("workflow_type".to_string(), workflow_name.to_string());
                meta
            },
        };

        let task_id = scheduler.submit_task(task).await?;
        task_ids.push(task_id);
    }

    // 启动调度器
    println!("启动集成调度器...");
    let scheduler_clone = scheduler.clone();
    tokio::spawn(async move {
        let _ = scheduler_clone.start().await;
    });

    // 等待所有任务完成
    println!("等待任务完成...");
    tokio::time::sleep(Duration::from_secs(8)).await;

    // 检查任务状态
    println!("任务状态:");
    for task_id in &task_ids {
        if let Some(status) = scheduler.get_task_status(*task_id).await {
            println!("  任务 {}: {:?}", task_id, status);
        }
    }

    // 显示调度器统计
    let stats = scheduler.get_stats().await;
    println!("调度器统计:");
    println!("  等待任务: {}", stats.pending_tasks);
    println!("  运行中任务: {}", stats.running_tasks);
    println!("  已完成任务: {}", stats.completed_tasks);
    println!("  失败任务: {}", stats.failed_tasks);
    println!("  总调度次数: {}", stats.total_scheduled);

    scheduler.stop().await?;
    println!("集成工作流演示完成\n");
    Ok(())
}

/// 演示错误恢复和重试机制
async fn demo_error_recovery() -> Result<()> {
    println!("=== 错误恢复演示 ===");

    let mut orchestrator = FlowOrchestrator::new();

    // 创建一个会失败的流程
    let failing_flow = FlowBuilder::new()
        .name("易失败流程")
        .step("risky_operation", |_ctx| {
            println!("  执行风险操作...");
            // 模拟 70% 的失败率
            if rand::random::<f64>() < 0.7 {
                Err(anyhow::anyhow!("操作失败"))
            } else {
                println!("  操作成功");
                Ok(())
            }
        })
        .build();

    // 创建备用流程
    let fallback_flow = FlowBuilder::new()
        .name("备用流程")
        .step("safe_operation", |_ctx| {
            println!("  执行安全备用操作...");
            Ok(())
        })
        .build();

    // 主节点使用重试和备用策略
    let main_node = FlowNode {
        id: "main".to_string(),
        name: "主要处理节点".to_string(),
        description: Some("可能失败的主要处理逻辑".to_string()),
        flow: Some(failing_flow),
        condition: None,
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::Fallback {
            fallback_flow_id: "fallback".to_string(),
        },
        timeout: Some(Duration::from_secs(5)),
        retry_config: Some(RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
        }),
    };

    // 备用节点
    let fallback_node = FlowNode {
        id: "fallback".to_string(),
        name: "备用处理节点".to_string(),
        description: Some("当主节点失败时的备用处理".to_string()),
        flow: Some(fallback_flow),
        condition: None,
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: Some(Duration::from_secs(3)),
        retry_config: None,
    };

    orchestrator.add_node(main_node).add_node(fallback_node);

    println!("开始执行错误恢复演示...");
    let results = orchestrator.execute_all().await?;

    println!("恢复结果:");
    for (node_id, _context) in results {
        println!("  节点 {} 已完成", node_id);
    }

    println!("错误恢复演示完成\n");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("FlowBuilder 调度器与编排器集成演示\n");

    // 运行各个演示
    demo_basic_scheduler().await?;
    demo_flow_orchestrator().await?;
    demo_integrated_workflow().await?;
    demo_error_recovery().await?;

    println!("所有演示完成！");
    Ok(())
}
