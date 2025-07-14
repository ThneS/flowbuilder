//! # FlowBuilder 新架构使用示例
//!
//! 演示新的分层架构：配置解析器 → 流程编排器 → 任务执行器

use flowbuilder_context::FlowContext;
use flowbuilder_yaml::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FlowBuilder 新架构演示 ===");

    // 1. 定义工作流配置
    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    ENVIRONMENT: "development"
    LOG_LEVEL: "info"
  vars:
    name: "新架构演示工作流"
    description: "展示分层架构的强大功能"
    max_retries: 3
  tasks:
    - task:
        id: "setup_task"
        name: "环境设置"
        description: "设置工作流执行环境"
        actions:
          - action:
              id: "setup_action"
              name: "环境初始化"
              description: "初始化执行环境"
              type: "builtin"
              flow:
                next: null
                retry:
                  max_retries: 2
                  delay: 1000
                timeout:
                  duration: 5000
              outputs:
                status: "initialized"
                timestamp: "2024-01-01T00:00:00Z"
              parameters:
                env:
                  value: "development"
                  required: true

    - task:
        id: "process_task"
        name: "数据处理"
        description: "处理业务数据"
        actions:
          - action:
              id: "process_action"
              name: "数据处理"
              description: "处理核心业务逻辑"
              type: "cmd"
              flow:
                next_if: "env.ENVIRONMENT == 'development'"
                next: null
                retry:
                  max_retries: 3
                  delay: 2000
                timeout:
                  duration: 10000
              outputs:
                processed_count: 100
                result: "success"
              parameters:
                input_path:
                  value: "/data/input"
                  required: true
                output_path:
                  value: "/data/output"
                  required: true

    - task:
        id: "notification_task"
        name: "通知发送"
        description: "发送处理结果通知"
        actions:
          - action:
              id: "notification_action"
              name: "发送通知"
              description: "发送邮件或消息通知"
              type: "http"
              flow:
                next: null
                timeout:
                  duration: 3000
              outputs:
                notification_sent: true
                recipients: 5
              parameters:
                endpoint:
                  value: "https://api.notification.com/send"
                  required: true
                message:
                  value: "工作流执行完成"
                  required: true
"#;

    // 2. 加载工作流配置
    println!("步骤1: 加载工作流配置");
    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    println!("  工作流版本: {}", config.workflow.version);
    println!("  任务数量: {}", config.workflow.tasks.len());

    // 3. 创建执行器
    println!("\n步骤2: 创建动态流程执行器");
    let mut executor = DynamicFlowExecutor::new(config)?;

    // 4. 获取工作流信息
    let workflow_info = executor.get_workflow_info();
    println!("  工作流名称: {}", workflow_info.name);
    println!("  环境变量数: {}", workflow_info.env_var_count);
    println!("  流程变量数: {}", workflow_info.flow_var_count);

    // 5. 验证工作流
    println!("\n步骤3: 验证工作流配置");
    executor.validate_workflow()?;
    println!("  配置验证通过！");

    // 6. 分析工作流复杂度
    println!("\n步骤4: 分析工作流复杂度");
    let complexity = executor.analyze_workflow_complexity()?;
    println!("  总节点数: {}", complexity.total_nodes);
    println!("  总阶段数: {}", complexity.total_phases);
    println!("  最大并行度: {}", complexity.max_parallel_nodes);
    println!("  条件节点数: {}", complexity.conditional_nodes);
    println!("  复杂度分数: {:.2}", complexity.complexity_score);

    // 7. 获取执行计划预览
    println!("\n步骤5: 生成执行计划预览");
    let execution_plan = executor.get_execution_plan_preview()?;
    println!("  执行阶段数: {}", execution_plan.phases.len());
    println!("  预计执行时间: {:?}", execution_plan.estimated_duration());

    for (i, phase) in execution_plan.phases.iter().enumerate() {
        println!(
            "  阶段 {}: {} ({:?})",
            i + 1,
            phase.name,
            phase.execution_mode
        );
        println!("    节点数: {}", phase.nodes.len());
        for node in &phase.nodes {
            println!("      - {}: {}", node.id, node.name);
        }
    }

    // 8. 创建执行上下文
    println!("\n步骤6: 创建执行上下文");
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    // 9. 执行工作流
    println!("\n步骤7: 执行工作流");
    println!("========================================");

    let result = executor.execute(context.clone()).await?;

    println!("========================================");
    println!("步骤8: 执行结果分析");
    println!(
        "  执行状态: {}",
        if result.success { "成功" } else { "失败" }
    );
    println!("  总执行时间: {:?}", result.total_duration);
    println!("  执行阶段数: {}", result.phase_results.len());

    let mut total_nodes = 0;
    let mut successful_nodes = 0;
    let mut failed_nodes = 0;

    for (i, phase_result) in result.phase_results.iter().enumerate() {
        println!("  阶段 {}: {}", i + 1, phase_result.phase_name);
        println!(
            "    状态: {}",
            if phase_result.success {
                "成功"
            } else {
                "失败"
            }
        );
        println!("    执行时间: {:?}", phase_result.duration);
        println!("    节点数: {}", phase_result.node_results.len());

        for node_result in &phase_result.node_results {
            total_nodes += 1;
            if node_result.success {
                successful_nodes += 1;
            } else {
                failed_nodes += 1;
            }

            println!(
                "      节点 {}: {} ({})",
                node_result.node_id,
                node_result.node_name,
                if node_result.success {
                    "成功"
                } else {
                    "失败"
                }
            );
            println!("        执行时间: {:?}", node_result.duration);
            if node_result.retry_count > 0 {
                println!("        重试次数: {}", node_result.retry_count);
            }
        }
    }

    // 10. 获取执行统计
    println!("\n步骤9: 执行统计");
    let stats = executor.get_stats();
    println!("  总任务数: {}", stats.total_tasks);
    println!("  成功任务数: {}", stats.successful_tasks);
    println!("  失败任务数: {}", stats.failed_tasks);
    println!("  平均执行时间: {:?}", stats.average_execution_time);

    // 11. 检查上下文状态
    println!("\n步骤10: 检查执行上下文");
    let context_guard = context.lock().await;
    println!(
        "  上下文状态: {}",
        if context_guard.ok { "正常" } else { "异常" }
    );
    println!("  错误数量: {}", context_guard.errors.len());
    println!("  步骤日志数: {}", context_guard.step_logs.len());
    println!("  变量数量: {}", context_guard.variables.len());

    // 显示一些变量
    for (key, value) in context_guard.variables.iter().take(5) {
        println!("    {}: {}", key, value);
    }

    drop(context_guard);

    println!("\n=== 新架构演示完成 ===");
    println!("架构特点：");
    println!("1. 配置解析器 - 负责从YAML解析出执行节点");
    println!("2. 流程编排器 - 负责分析依赖关系并生成执行计划");
    println!("3. 任务执行器 - 负责按计划执行具体任务");
    println!("4. 分层清晰 - 每层职责明确，易于维护和扩展");
    println!("5. 高性能 - 支持并行执行和智能调度");

    Ok(())
}
