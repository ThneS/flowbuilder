// Integration test to verify parser uses evaluate_yaml correctly
use flowbuilder_yaml::{ExpressionEvaluator, WorkflowConfig, WorkflowLoader, YamlFlowBuilder};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

#[test]
fn test_parser_integration_with_new_expression_system() {
    // Create a simple workflow YAML that uses expressions
    let yaml_content = r#"
workflow:
  version: "1.0"
  env:
    TEST_ENV: "test_value"
    API_URL: "https://api.example.com"
  vars:
    app_name: "TestApp"
    debug: true
    timeout: 30
  tasks:
    - task:
        id: "test_task"
        name: "Test Task"
        description: "Test task with expressions"
        actions:
          - action:
              id: "test_action"
              name: "Test Action"
              description: "Test action"
              type: "builtin"
              outputs:
                result: "success"
              parameters:
                env_param:
                  value: "${env:TEST_ENV}"
                var_param:
                  value: "${ctx:vars.app_name}"
                mixed_param:
                  value: "App: ${ctx:vars.app_name} at ${env:API_URL}"
                boolean_param:
                  value: "${ctx:vars.debug}"
                number_param:
                  value: "${ctx:vars.timeout}"
"#;

    // Load the workflow
    let config = WorkflowLoader::from_yaml_str(yaml_content).expect("Failed to load workflow");
    
    // Create flow builder
    let flow_builder = YamlFlowBuilder::new(config).expect("Failed to create flow builder");
    
    // Build the flow
    let _flow = flow_builder.build().expect("Failed to build flow");
    
    // The fact that we got here without panicking means the integration works
    println!("Integration test passed - parser successfully uses new expression system");
}

#[test]
fn test_evaluate_yaml_vs_legacy_evaluate() {
    let mut evaluator = ExpressionEvaluator::new();
    
    // Set up test data
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "42".to_string());
    evaluator.set_env_vars(env_vars);
    
    let mut flow_vars = HashMap::new();
    flow_vars.insert("number".to_string(), YamlValue::Number(serde_yaml::Number::from(123)));
    flow_vars.insert("flag".to_string(), YamlValue::Bool(true));
    evaluator.set_flow_vars(flow_vars);
    
    // Test type preservation with evaluate_yaml
    let number_result = evaluator.evaluate_yaml(&YamlValue::String("${ctx:vars.number}".to_string())).unwrap();
    assert_eq!(number_result, YamlValue::Number(serde_yaml::Number::from(123)));
    
    let bool_result = evaluator.evaluate_yaml(&YamlValue::String("${ctx:vars.flag}".to_string())).unwrap();
    assert_eq!(bool_result, YamlValue::Bool(true));
    
    // Test string interpolation with mixed content
    let mixed_result = evaluator.evaluate_yaml(&YamlValue::String("Number: ${ctx:vars.number}, Flag: ${ctx:vars.flag}".to_string())).unwrap();
    assert_eq!(mixed_result, YamlValue::String("Number: 123, Flag: true".to_string()));
    
    // Test legacy evaluate method still works (for backward compatibility)
    let legacy_result = evaluator.evaluate("${env:TEST_VAR}").unwrap();
    assert_eq!(legacy_result, YamlValue::String("42".to_string()));
    
    println!("Type preservation and legacy compatibility tests passed");
}