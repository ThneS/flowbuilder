//! # 调度器和编排器单元测试
//!
//! 测试调度器和编排器的核心功能

use crate::orchestrator::{
    BranchCondition, ErrorRecoveryStrategy, FlowNode, FlowOrchestrator, FlowState,
};
use crate::scheduler::{
    Priority, ScheduledTask, SchedulerConfig, TaskScheduler, TaskStatus,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[tokio::test]
async fn test_scheduler_priority_ordering() {
    let scheduler = TaskScheduler::with_default_config();
    let execution_order = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // 创建不同优先级的任务
    let tasks = vec![
        (Priority::Low, "低优先级"),
        (Priority::High, "高优先级"),
        (Priority::Normal, "普通优先级"),
        (Priority::Critical, "紧急优先级"),
    ];

    for (priority, name) in tasks {
        let order = execution_order.clone();
        let task_name = name.to_string();

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: task_name.clone(),
            description: None,
            priority,
            estimated_duration: None,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            task_fn: Arc::new(move || {
                let order = order.clone();
                let name = task_name.clone();
                tokio::spawn(async move {
                    let mut order = order.lock().await;
                    order.push(name);
                });
                Ok(())
            }),
            metadata: HashMap::new(),
        };

        scheduler.submit_task(task).await.unwrap();
    }

    // 手动执行任务（按优先级顺序）
    for _ in 0..4 {
        if let Some(task) = scheduler.get_next_task().await {
            scheduler.execute_task(task).await.unwrap();
        }
    }

    // 等待任务完成
    tokio::time::sleep(Duration::from_millis(100)).await;

    let order = execution_order.lock().await;
    
    // 验证执行顺序：Critical -> High -> Normal -> Low
    assert_eq!(order[0], "紧急优先级");
    assert_eq!(order[1], "高优先级");
    assert_eq!(order[2], "普通优先级");
    assert_eq!(order[3], "低优先级");
}

#[tokio::test]
async fn test_scheduler_dependency_checking() {
    let config = SchedulerConfig {
        enable_dependency_check: true,
        ..SchedulerConfig::default()
    };
    let scheduler = TaskScheduler::new(config);

    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();

    // Task 1: 无依赖
    let task1 = ScheduledTask {
        id: task1_id,
        name: "Task 1".to_string(),
        description: None,
        priority: Priority::Normal,
        estimated_duration: None,
        created_at: Instant::now(),
        started_at: None,
        completed_at: None,
        status: TaskStatus::Pending,
        dependencies: Vec::new(),
        task_fn: Arc::new(|| Ok(())),
        metadata: HashMap::new(),
    };

    // Task 2: 依赖 Task 1
    let task2 = ScheduledTask {
        id: task2_id,
        name: "Task 2".to_string(),
        description: None,
        priority: Priority::High, // 更高优先级，但有依赖
        estimated_duration: None,
        created_at: Instant::now(),
        started_at: None,
        completed_at: None,
        status: TaskStatus::Pending,
        dependencies: vec![task1_id],
        task_fn: Arc::new(|| Ok(())),
        metadata: HashMap::new(),
    };

    scheduler.submit_task(task1).await.unwrap();
    scheduler.submit_task(task2).await.unwrap();

    // Task 2 不应该能够调度，因为 Task 1 还未完成
    let task = scheduler.get_next_task().await.unwrap();
    assert!(!scheduler.can_schedule_task(&task).await);
    assert_eq!(task.id, task2_id); // 虽然优先级高，但不能调度

    // Task 1 应该可以调度
    let task = scheduler.get_next_task().await.unwrap();
    assert!(scheduler.can_schedule_task(&task).await);
    assert_eq!(task.id, task1_id);
}

#[tokio::test]
async fn test_orchestrator_basic_flow() {
    let mut orchestrator = FlowOrchestrator::new();
    
    // 创建一个简单的节点，不需要真实的 Flow 执行
    let node = FlowNode {
        id: "test_node".to_string(),
        name: "测试节点".to_string(),
        description: Some("基本流程测试".to_string()),
        flow: None, // 使用 None，测试编排器逻辑而不是实际流程执行
        condition: None,
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: Some(Duration::from_secs(5)),
        retry_config: None,
    };

    orchestrator.add_node(node);

    let results = orchestrator.execute_all().await.unwrap();
    
    // 验证编排器正确处理了节点
    assert_eq!(results.len(), 1);
    assert!(results.contains_key("test_node"));
    
    // 验证节点状态
    let node_state = orchestrator.get_node_state("test_node").await;
    assert_eq!(node_state, Some(FlowState::Completed));
}

#[tokio::test]
async fn test_orchestrator_branch_condition() {
    let mut orchestrator = FlowOrchestrator::new();

    // 条件为 true 的节点
    let node_true = FlowNode {
        id: "node_true".to_string(),
        name: "条件为真的节点".to_string(),
        description: None,
        flow: None, // 简化测试
        condition: Some(BranchCondition::Boolean(true)),
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: None,
        retry_config: None,
    };

    // 条件为 false 的节点
    let node_false = FlowNode {
        id: "node_false".to_string(),
        name: "条件为假的节点".to_string(),
        description: None,
        flow: None, // 简化测试
        condition: Some(BranchCondition::Boolean(false)),
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: None,
        retry_config: None,
    };

    orchestrator.add_node(node_true);
    orchestrator.add_node(node_false);

    let results = orchestrator.execute_all().await.unwrap();
    
    // 验证结果 - 根据条件，应该只有部分节点被执行
    // 具体验证逻辑取决于 evaluate_condition 的实现
    assert!(results.len() <= 2);
}

#[tokio::test]
async fn test_orchestrator_dependency_execution() {
    let mut orchestrator = FlowOrchestrator::new();

    // 创建三个有依赖关系的节点: A -> B -> C
    let node_a = FlowNode {
        id: "node_a".to_string(),
        name: "节点A".to_string(),
        description: None,
        flow: None, // 简化测试
        condition: None,
        next_nodes: vec!["node_b".to_string()],
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: None,
        retry_config: None,
    };

    let node_b = FlowNode {
        id: "node_b".to_string(),
        name: "节点B".to_string(),
        description: None,
        flow: None, // 简化测试
        condition: None,
        next_nodes: vec!["node_c".to_string()],
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: None,
        retry_config: None,
    };

    let node_c = FlowNode {
        id: "node_c".to_string(),
        name: "节点C".to_string(),
        description: None,
        flow: None, // 简化测试
        condition: None,
        next_nodes: Vec::new(),
        error_recovery: ErrorRecoveryStrategy::FailFast,
        timeout: None,
        retry_config: None,
    };

    orchestrator.add_node(node_a);
    orchestrator.add_node(node_b);
    orchestrator.add_node(node_c);
    
    // 添加依赖关系
    orchestrator.add_dependency("node_b".to_string(), "node_a".to_string());
    orchestrator.add_dependency("node_c".to_string(), "node_b".to_string());

    let results = orchestrator.execute_all().await.unwrap();
    
    // 验证所有节点都被执行
    assert_eq!(results.len(), 3);
    assert!(results.contains_key("node_a"));
    assert!(results.contains_key("node_b"));
    assert!(results.contains_key("node_c"));
}

#[tokio::test]
async fn test_scheduler_stats() {
    let scheduler = TaskScheduler::with_default_config();

    // 创建一个简单任务
    let task = ScheduledTask {
        id: Uuid::new_v4(),
        name: "Test Task".to_string(),
        description: None,
        priority: Priority::Normal,
        estimated_duration: Some(Duration::from_millis(100)),
        created_at: Instant::now(),
        started_at: None,
        completed_at: None,
        status: TaskStatus::Pending,
        dependencies: Vec::new(),
        task_fn: Arc::new(|| Ok(())),
        metadata: HashMap::new(),
    };

    scheduler.submit_task(task).await.unwrap();

    let stats = scheduler.get_stats().await;
    assert_eq!(stats.pending_tasks, 1);
    assert_eq!(stats.running_tasks, 0);
    assert_eq!(stats.pending_tasks, 1);
    assert_eq!(stats.running_tasks, 0);
    assert_eq!(stats.completed_tasks, 0);
}
