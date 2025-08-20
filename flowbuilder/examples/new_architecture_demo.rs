//! # FlowBuilder 新架构使用示例
//!
//! 演示新的分层架构：配置解析器 → 流程编排器 → 任务执行器

use flowbuilder_context::FlowContext;
use flowbuilder_yaml::prelude::*;
use std::sync::Arc;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    flowbuilder::logging::init();
    info!("=== FlowBuilder 新架构演示 ===");

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
                next: setup_task
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
                next: setup_task
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
    info!("步骤1: 加载工作流配置");
    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    info!(version = %config.workflow.version, tasks = config.workflow.tasks.len(), "配置加载完成");

    // 3. 创建执行器
    info!("步骤2: 创建动态流程执行器");
    let mut executor = DynamicFlowExecutor::new(config)?;
    #[cfg(feature = "runtime")]
    executor.set_print_plan(true);

    // 4. 获取工作流信息
    let workflow_info = executor.get_workflow_info();
    info!(name = %workflow_info.name, env_vars = workflow_info.env_var_count, flow_vars = workflow_info.flow_var_count, "工作流信息");

    // 5. 验证工作流
    info!("步骤3: 验证工作流配置");
    #[cfg(feature = "runtime")]
    executor.validate_workflow()?;
    info!("配置验证通过");

    // 6. 分析工作流复杂度
    info!("步骤4: 分析工作流复杂度");
    #[cfg(feature = "runtime")]
    {
        let complexity = executor.analyze_workflow_complexity()?;
        info!(
            total_nodes = complexity.total_nodes,
            total_phases = complexity.total_phases,
            max_parallel = complexity.max_parallel_nodes,
            conditional_nodes = complexity.conditional_nodes,
            score = complexity.complexity_score,
            "复杂度"
        );
    }

    // 7. 获取执行计划预览
    info!("步骤5: 生成执行计划预览");
    #[cfg(feature = "runtime")]
    {
        let pretty = executor.print_execution_plan()?;
        debug!(plan_pretty = %pretty, "执行计划预览");
    }

    // 8. 创建执行上下文
    info!("步骤6: 创建执行上下文");
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    // 9. 执行工作流
    info!("步骤7: 执行工作流");

    #[cfg(feature = "runtime")]
    let result = executor.execute(context.clone()).await?;

    info!("步骤8: 执行结果分析");
    #[cfg(feature = "runtime")]
    {
        info!(success = result.success, total_duration_ms = ?result.total_duration, phases = result.phase_results.len(), "执行结果");
    }

    let mut total_nodes = 0;
    let mut successful_nodes = 0;
    let mut failed_nodes = 0;

    #[cfg(feature = "runtime")]
    for (i, phase_result) in result.phase_results.iter().enumerate() {
        info!(phase_index = i + 1, phase_name = %phase_result.phase_name, success = phase_result.success, duration_ms = ?phase_result.duration, node_count = phase_result.node_results.len(), "阶段结果");

        for node_result in &phase_result.node_results {
            total_nodes += 1;
            if node_result.success {
                successful_nodes += 1;
            } else {
                failed_nodes += 1;
            }

            info!(node_id = %node_result.node_id, node_name = %node_result.node_name, success = node_result.success, duration_ms = ?node_result.duration, retry_count = node_result.retry_count, "节点结果");
        }
    }

    // 输出节点统计信息
    #[cfg(feature = "runtime")]
    {
        info!(
            total_nodes = total_nodes,
            success_nodes = successful_nodes,
            failed_nodes = failed_nodes,
            "节点执行统计"
        );
    }

    // 10. 获取执行统计
    info!("步骤9: 执行统计");
    #[cfg(all(feature = "runtime", feature = "perf-metrics"))]
    {
        let stats = executor.get_stats();
        info!(total_tasks = stats.total_tasks, success_tasks = stats.successful_tasks, failed_tasks = stats.failed_tasks, avg_duration_ms = ?stats.average_execution_time, "执行统计");
    }

    // 11. 检查上下文状态
    info!("步骤10: 检查执行上下文");
    let context_guard = context.lock().await;
    info!(
        ok = context_guard.ok,
        errors = context_guard.errors.len(),
        step_logs = context_guard.step_logs.len(),
        vars = context_guard.variables.len(),
        "上下文状态"
    );

    // 显示一些变量
    for (key, value) in context_guard.variables.iter().take(5) {
        debug!(key = %key, value = %value, "变量样例");
    }

    drop(context_guard);

    info!("=== 新架构演示完成 ===");
    info!("架构特点：1. 配置解析器 2. 流程编排器 3. 任务执行器 4. 分层清晰 5. 高性能");

    Ok(())
}
