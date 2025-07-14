use anyhow::Result;
use flowbuilder_context::FlowContext;
use flowbuilder_yaml::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== FlowBuilder 动态 YAML 加载演示 ===\n");

    // 演示1：从 YAML 字符串加载简单工作流
    demo_simple_yaml_flow().await?;

    // 演示2：从文件加载复杂工作流
    demo_complex_yaml_flow().await?;

    // 演示3：动态表达式求值
    demo_expression_evaluation().await?;

    // 演示4：条件流程控制
    demo_conditional_flow().await?;

    // 演示5：错误处理和重试
    demo_error_handling().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

async fn demo_simple_yaml_flow() -> Result<()> {
    println!("1. 简单 YAML 工作流演示:");

    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    ENVIRONMENT: "demo"
  vars:
    name: "简单演示工作流"
    description: "这是一个简单的演示"
  tasks:
    - task:
        id: "greeting_task"
        name: "问候任务"
        description: "执行问候"
        actions:
          - action:
              id: "say_hello"
              name: "说你好"
              description: "打印问候信息"
              type: "builtin"
              outputs:
                message: "Hello, FlowBuilder!"
                timestamp: "2025-01-06"
              parameters:
                greeting:
                  value: "${{ vars.name }}"
                  required: true
"#;

    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    WorkflowLoader::validate(&config)?;

    let mut executor = DynamicFlowExecutor::new(config)?;
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    executor.execute(context).await?;
    println!();
    Ok(())
}

async fn demo_complex_yaml_flow() -> Result<()> {
    println!("2. 复杂 YAML 工作流演示:");

    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    API_URL: "https://api.example.com"
    ENVIRONMENT: "production"
  vars:
    name: "数据处理工作流"
    batch_size: 100
    retry_enabled: true
  template:
    id: "data_processing_template"
    name: "数据处理模板"
    version: "1.0"
    description: "用于数据处理的基础模板"
  tasks:
    - task:
        id: "data_fetch"
        name: "数据获取"
        description: "从外部源获取数据"
        actions:
          - action:
              id: "fetch_api_data"
              name: "获取 API 数据"
              description: "从 API 获取数据"
              type: "http"
              flow:
                next: "data_process.validate_data"
                retry:
                  max_retries: 3
                  delay: 1000
                timeout:
                  duration: 5000
                on_error: "error_handler"
                on_timeout: "timeout_handler"
              outputs:
                status: 200
                data: "sample_data"
                record_count: 1000
              parameters:
                url:
                  value: "${{ env.API_URL }}/data"
                  required: true
                method:
                  value: "GET"
                  required: true
    - task:
        id: "data_process"
        name: "数据处理"
        description: "处理和验证数据"
        actions:
          - action:
              id: "validate_data"
              name: "验证数据"
              description: "验证获取的数据"
              type: "builtin"
              flow:
                next: "data_transform.transform_records"
                next_if: "${data_fetch.fetch_api_data.outputs.status} == 200"
                retry:
                  max_retries: 2
                  delay: 500
              outputs:
                validation_status: "passed"
                error_count: 0
              parameters:
                data:
                  value: "${data_fetch.fetch_api_data.outputs.data}"
                  required: true
    - task:
        id: "data_transform"
        name: "数据转换"
        description: "转换数据格式"
        actions:
          - action:
              id: "transform_records"
              name: "转换记录"
              description: "将数据转换为目标格式"
              type: "cmd"
              flow:
                next: null
                while_util:
                  condition: "${data_process.validate_data.outputs.error_count} == 0"
                  max_iterations: 5
              outputs:
                transformed_count: 1000
                output_format: "json"
              parameters:
                input_data:
                  value: "${data_process.validate_data.outputs.validation_status}"
                  required: true
                batch_size:
                  value: "${{ vars.batch_size }}"
                  required: false
"#;

    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    WorkflowLoader::validate(&config)?;

    let mut executor = DynamicFlowExecutor::new(config)?;
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    executor.execute(context).await?;
    println!();
    Ok(())
}

async fn demo_expression_evaluation() -> Result<()> {
    println!("3. 表达式求值演示:");

    let mut evaluator = ExpressionEvaluator::new();

    // 设置环境变量
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("APP_NAME".to_string(), "FlowBuilder".to_string());
    env_vars.insert("VERSION".to_string(), "1.0".to_string());
    evaluator.set_env_vars(env_vars);

    // 设置流程变量
    let mut flow_vars = std::collections::HashMap::new();
    flow_vars.insert("enabled".to_string(), serde_yaml::Value::Bool(true));
    flow_vars.insert(
        "max_retries".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(3)),
    );
    evaluator.set_flow_vars(flow_vars);

    // 设置上下文变量
    evaluator.set_context_var(
        "task1.outputs.status",
        serde_yaml::Value::Number(serde_yaml::Number::from(200)),
    );

    // 测试各种表达式
    let expressions = vec![
        "${{ env.APP_NAME }}",
        "${{ env.VERSION }}",
        "${{ vars.enabled }}",
        "${{ vars.max_retries }}",
        "${task1.outputs.status}",
    ];

    for expr in expressions {
        match evaluator.evaluate(expr) {
            Ok(result) => println!("  {expr} => {result:?}"),
            Err(e) => println!("  {expr} => 错误: {e}"),
        }
    }

    // 测试条件表达式
    let conditions = vec![
        "true",
        "false",
        "test == test",
        "hello != world",
        "true && true",
        "false || true",
    ];

    println!("\n  条件求值测试:");
    for condition in conditions {
        match evaluator.evaluate_condition(condition) {
            Ok(result) => println!("    {condition} => {result}"),
            Err(e) => println!("    {condition} => 错误: {e}"),
        }
    }

    println!();
    Ok(())
}

async fn demo_conditional_flow() -> Result<()> {
    println!("4. 条件流程控制演示:");

    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    ENVIRONMENT: "production"
  vars:
    deploy_enabled: true
    skip_tests: false
  tasks:
    - task:
        id: "build_task"
        name: "构建任务"
        description: "构建应用程序"
        actions:
          - action:
              id: "compile_code"
              name: "编译代码"
              description: "编译源代码"
              type: "cmd"
              outputs:
                build_status: "success"
                artifacts: "app.tar.gz"
              parameters: {}
    - task:
        id: "test_task"
        name: "测试任务"
        description: "运行测试"
        actions:
          - action:
              id: "run_tests"
              name: "运行测试"
              description: "执行单元测试"
              type: "cmd"
              flow:
                next_if: "${{ vars.skip_tests }} == false"
                next: "deploy_task.deploy_app"
              outputs:
                test_status: "passed"
                coverage: "95%"
              parameters: {}
    - task:
        id: "deploy_task"
        name: "部署任务"
        description: "部署应用程序"
        actions:
          - action:
              id: "deploy_app"
              name: "部署应用"
              description: "将应用部署到生产环境"
              type: "cmd"
              flow:
                next_if: "${{ vars.deploy_enabled }} == true"
                next: null
              outputs:
                deploy_status: "deployed"
                url: "https://app.example.com"
              parameters:
                environment:
                  value: "${{ env.ENVIRONMENT }}"
                  required: true
"#;

    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    let mut executor = DynamicFlowExecutor::new(config)?;
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    executor.execute(context).await?;
    println!();
    Ok(())
}

async fn demo_error_handling() -> Result<()> {
    println!("5. 错误处理和重试演示:");

    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    SERVICE_URL: "https://api.example.com"
  vars:
    name: "错误处理演示"
    max_attempts: 3
  tasks:
    - task:
        id: "unreliable_task"
        name: "不可靠任务"
        description: "可能失败的任务"
        actions:
          - action:
              id: "flaky_service_call"
              name: "不稳定服务调用"
              description: "调用可能失败的外部服务"
              type: "http"
              flow:
                retry:
                  max_retries: 3
                  delay: 1000
                timeout:
                  duration: 3000
                on_error: "error_task.handle_error"
                on_timeout: "timeout_task.handle_timeout"
              outputs:
                status: 200
                data: "response_data"
              parameters:
                url:
                  value: "${{ env.SERVICE_URL }}/unreliable"
                  required: true
    - task:
        id: "error_task"
        name: "错误处理任务"
        description: "处理错误情况"
        actions:
          - action:
              id: "handle_error"
              name: "处理错误"
              description: "记录和处理错误"
              type: "builtin"
              outputs:
                error_logged: true
                fallback_data: "default_value"
              parameters: {}
    - task:
        id: "timeout_task"
        name: "超时处理任务"
        description: "处理超时情况"
        actions:
          - action:
              id: "handle_timeout"
              name: "处理超时"
              description: "记录和处理超时"
              type: "builtin"
              outputs:
                timeout_logged: true
                retry_scheduled: true
              parameters: {}
"#;

    let config = WorkflowLoader::from_yaml_str(yaml_content)?;
    let mut executor = DynamicFlowExecutor::new(config)?;
    let context = Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

    executor.execute(context).await?;
    println!();
    Ok(())
}
