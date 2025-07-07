//! # YAML Runtime 集成示例
//!
//! 展示如何使用 YAML 配置结合 runtime 功能

use flowbuilder_context::FlowContext;
use flowbuilder_yaml::{WorkflowLoader, YamlRuntimeIntegrator, convenience};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== FlowBuilder YAML Runtime 集成示例 ===\n");

    // 示例 YAML 配置
    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    FLOWBUILDER_ENV: "production"
    FLOWBUILDER_MAX_CONCURRENT: "8"
    FLOWBUILDER_STRATEGY: "priority"
    FLOWBUILDER_VERBOSE: "true"
  vars:
    name: "数据处理工作流"
    description: "演示 Runtime 集成的数据处理流程"
  tasks:
    - task:
        id: "data_fetch"
        name: "数据获取任务"
        description: "从外部源获取数据"
        actions:
          - action:
              id: "fetch_action"
              name: "获取数据"
              type: "http"
              parameters:
                url: "https://api.example.com/data"
                method: "GET"
              outputs:
                data: "{{response.body}}"
                status: "{{response.status}}"

    - task:
        id: "data_process"
        name: "high优先级数据处理任务"
        description: "处理获取的数据"
        actions:
          - action:
              id: "process_action"
              name: "处理数据"
              type: "builtin"
              parameters:
                input: "{{data_fetch.outputs.data}}"
              outputs:
                processed_data: "{{processed_result}}"
                count: 100

    - task:
        id: "data_save"
        name: "数据保存任务"
        description: "保存处理后的数据"
        actions:
          - action:
              id: "save_action"
              name: "保存数据"
              type: "cmd"
              parameters:
                command: "save"
                target: "/tmp/result.json"
                data: "{{data_process.outputs.processed_data}}"
              outputs:
                saved: true
                path: "/tmp/result.json"

    - task:
        id: "critical_cleanup"
        name: "critical优先级清理任务"
        description: "清理临时文件"
        actions:
          - action:
              id: "cleanup_action"
              name: "清理"
              type: "cmd"
              parameters:
                command: "cleanup"
              outputs:
                cleaned: true
"#;

    // 1. 基本 YAML 加载和验证
    println!("1. 加载和验证 YAML 配置...");
    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    WorkflowLoader::validate(&config)?;
    println!("   ✓ YAML 配置验证通过\n");

    // 2. 创建 Runtime 集成器
    println!("2. 创建 Runtime 集成器...");
    let integrator = YamlRuntimeIntegrator::new(config.clone())?;
    println!("   ✓ Runtime 集成器创建成功\n");

    // 3. 创建执行上下文
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    // 4. 演示调度器执行
    println!("3. 使用调度器执行工作流...");
    let scheduling_results = integrator.execute_with_scheduling(context.clone()).await?;
    println!(
        "   ✓ 调度执行完成，处理了 {} 个任务",
        scheduling_results.len()
    );
    for (task_id, status) in &scheduling_results {
        println!("     任务 {}: {:?}", task_id, status);
    }
    println!();

    // 5. 演示编排器执行
    println!("4. 使用编排器执行工作流...");
    let orchestration_results = integrator
        .execute_with_orchestration(context.clone())
        .await?;
    println!(
        "   ✓ 编排执行完成，处理了 {} 个节点",
        orchestration_results.len()
    );
    for (node_id, _) in &orchestration_results {
        println!("     节点 {}: 执行完成", node_id);
    }
    println!();

    // 6. 演示混合执行模式
    println!("5. 使用混合模式执行工作流...");
    let (sched_results, orch_results) = integrator.execute_hybrid(context.clone()).await?;
    println!("   ✓ 混合执行完成");
    println!("     调度结果: {} 个任务", sched_results.len());
    println!("     编排结果: {} 个节点", orch_results.len());
    println!();

    // 7. 使用便捷函数
    println!("6. 使用便捷函数执行...");

    // 调度器便捷函数
    let quick_sched_results =
        convenience::quick_execute_with_scheduler(config.clone(), context.clone()).await?;
    println!(
        "   ✓ 快速调度执行完成，{} 个任务",
        quick_sched_results.len()
    );

    // 编排器便捷函数
    let quick_orch_results =
        convenience::quick_execute_with_orchestrator(config.clone(), context.clone()).await?;
    println!("   ✓ 快速编排执行完成，{} 个节点", quick_orch_results.len());

    // 混合便捷函数
    let (quick_sched, quick_orch) =
        convenience::quick_execute_hybrid(config.clone(), context.clone()).await?;
    println!(
        "   ✓ 快速混合执行完成，{} 个任务，{} 个节点",
        quick_sched.len(),
        quick_orch.len()
    );
    println!();

    // 8. 演示任务优先级识别
    println!("7. 任务优先级和配置分析...");
    let tasks = integrator.convert_to_scheduled_tasks()?;
    for task in &tasks {
        println!("   任务: {}", task.name);
        println!("     优先级: {:?}", task.priority);
        println!("     预估时间: {:?}", task.estimated_duration);
        println!("     元数据: {:?}", task.metadata);
        println!();
    }

    // 9. 演示流程节点转换
    println!("8. 流程节点转换...");
    let nodes = integrator.convert_to_flow_nodes()?;
    for node in &nodes {
        println!("   节点: {} - {}", node.id, node.name);
        println!("     描述: {:?}", node.description);
        println!("     错误恢复: {:?}", node.error_recovery);
        println!("     超时: {:?}", node.timeout);
        println!();
    }

    println!("=== 所有示例执行完成 ===");
    Ok(())
}
