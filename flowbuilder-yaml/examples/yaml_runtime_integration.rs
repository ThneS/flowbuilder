//! # YAML Runtime 集成示例
//!
//! 演示如何使用 YAML 配置与 flowbuilder-runtime 的集成功能

use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_yaml::{convenience, YamlRuntimeIntegrator};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 YAML Runtime 集成示例");

    // YAML 工作流配置
    let yaml_content = r#"
name: yaml_runtime_integration_demo
version: 1.0.0
description: "演示 YAML 与 runtime 集成的工作流"

vars:
  environment: "development"
  max_retries: 3

workflow:
  version: "1.0.0"
  tasks:
    - task:
        id: "data_processing"
        name: "数据处理任务"
        description: "处理输入数据"
        priority: high
        retry:
          max_attempts: 3
          delay: 2s
        timeout: 30s
        actions:
          - action:
              id: "load_data"
              name: "加载数据"
              description: "从数据库加载数据"
              type: builtin
              parameters:
                source:
                  value: "database"
                  required: true
                limit:
                  value: 100
                  required: false
              outputs:
                data: "${result.rows}"
          - action:
              id: "transform_data"
              name: "转换数据"
              description: "转换数据格式"
              type: builtin
              parameters:
                input:
                  value: "${data}"
                  required: true
                format:
                  value: "json"
                  required: false
              outputs:
                transformed: "${result.transformed}"

    - task:
        id: "notification"
        name: "通知任务"
        description: "发送处理完成通知"
        priority: medium
        depends_on:
          - "data_processing"
        actions:
          - action:
              id: "send_notification"
              name: "发送通知"
              description: "发送处理完成通知"
              type: builtin
              parameters:
                recipient:
                  value: "admin@example.com"
                  required: true
                message:
                  value: "数据处理完成: ${transformed}"
                  required: true
              outputs:
                status: "${result.status}"
"#;

    // 1. 使用便捷函数执行 YAML 工作流
    println!("\n📋 1. 使用便捷函数执行工作流");
    let context = convenience::create_flow_context();
    match convenience::execute_yaml_workflow_with_scheduling(yaml_content, context.clone()).await {
        Ok(results) => {
            println!("✅ 调度执行完成，处理了 {} 个任务", results.len());
            for (task_id, status) in results {
                println!("   - 任务 {}: {:?}", task_id, status);
            }
        }
        Err(e) => println!("❌ 调度执行失败: {}", e),
    }

    // 2. 使用 YamlRuntimeIntegrator 直接操作
    println!("\n🔧 2. 使用集成器执行工作流");
    match YamlRuntimeIntegrator::from_yaml_str(yaml_content) {
        Ok(integrator) => {
            let context = convenience::create_flow_context();
            match integrator.execute_with_orchestration(context).await {
                Ok(results) => {
                    println!("✅ 编排执行完成，处理了 {} 个节点", results.len());
                    for (node_id, flow_context) in results {
                        println!(
                            "   - 节点 {}: {} 变量",
                            node_id,
                            flow_context.variables.len()
                        );
                    }
                }
                Err(e) => println!("❌ 编排执行失败: {}", e),
            }
        }
        Err(e) => println!("❌ 创建集成器失败: {}", e),
    }

    // 3. 演示混合执行模式
    println!("\n⚡ 3. 演示混合执行");
    match convenience::execute_yaml_workflow_mixed(yaml_content).await {
        Ok(context) => {
            println!("✅ 混合执行完成");
            println!("   - 最终上下文包含 {} 个变量", context.variables.len());
            for (key, value) in context.variables.iter() {
                println!("     {}={}", key, value);
            }
        }
        Err(e) => println!("❌ 混合执行失败: {}", e),
    }

    println!("\n🎉 所有示例执行完成！");
    Ok(())
}
