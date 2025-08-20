//! Demo showcasing the implemented execute_action_by_type functionality

use flowbuilder_core::ActionSpec;
use flowbuilder_runtime::EnhancedTaskExecutor;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let context = Arc::new(tokio::sync::Mutex::new(
        flowbuilder_context::FlowContext::default(),
    ));

    println!("🚀 FlowBuilder Action Demo");
    println!("==========================");

    // Demo 1: Builtin set_variable action
    println!("\n📝 Demo 1: Setting a variable using builtin action");
    let set_var_action = ActionSpec {
        action_type: "builtin".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), serde_yaml::Value::String("set_variable".to_string()));
            params.insert("key".to_string(), serde_yaml::Value::String("demo_var".to_string()));
            params.insert("value".to_string(), serde_yaml::Value::String("Hello FlowBuilder!".to_string()));
            params
        },
        outputs: HashMap::new(),
    };

    match EnhancedTaskExecutor::execute_action_by_type(&set_var_action, context.clone()).await {
        Ok(_) => println!("✅ Variable set successfully"),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demo 2: Builtin log action
    println!("\n📊 Demo 2: Logging a message using builtin action");
    let log_action = ActionSpec {
        action_type: "builtin".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("operation".to_string(), serde_yaml::Value::String("log".to_string()));
            params.insert("message".to_string(), serde_yaml::Value::String("This is a demo log message".to_string()));
            params.insert("level".to_string(), serde_yaml::Value::String("info".to_string()));
            params
        },
        outputs: HashMap::new(),
    };

    match EnhancedTaskExecutor::execute_action_by_type(&log_action, context.clone()).await {
        Ok(_) => println!("✅ Log message sent successfully"),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demo 3: Command execution
    println!("\n💻 Demo 3: Executing a command");
    let cmd_action = ActionSpec {
        action_type: "cmd".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("command".to_string(), serde_yaml::Value::String("echo".to_string()));
            params.insert("args".to_string(), serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("Hello from command execution!".to_string())
            ]));
            params
        },
        outputs: HashMap::new(),
    };

    match EnhancedTaskExecutor::execute_action_by_type(&cmd_action, context.clone()).await {
        Ok(_) => {
            println!("✅ Command executed successfully");
            let guard = context.lock().await;
            if let Some(stdout) = guard.variables.get("cmd_stdout") {
                println!("📤 Command output: {}", stdout);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demo 4: WASM action (simulated)
    println!("\n🕸️ Demo 4: WASM action execution (simulated)");
    let wasm_action = ActionSpec {
        action_type: "wasm".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("module".to_string(), serde_yaml::Value::String("demo.wasm".to_string()));
            params.insert("function".to_string(), serde_yaml::Value::String("demo_function".to_string()));
            params
        },
        outputs: HashMap::new(),
    };

    match EnhancedTaskExecutor::execute_action_by_type(&wasm_action, context.clone()).await {
        Ok(_) => println!("✅ WASM action executed successfully (simulated)"),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demo 5: Composite action
    println!("\n🔗 Demo 5: Composite action execution");
    let composite_action = ActionSpec {
        action_type: "composite".to_string(),
        parameters: {
            let mut params = HashMap::new();
            
            let sub_actions = vec![
                serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::mapping::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("type".to_string()),
                        serde_yaml::Value::String("builtin".to_string())
                    );
                    map.insert(
                        serde_yaml::Value::String("parameters".to_string()),
                        serde_yaml::Value::Mapping({
                            let mut param_map = serde_yaml::mapping::Mapping::new();
                            param_map.insert(
                                serde_yaml::Value::String("operation".to_string()),
                                serde_yaml::Value::String("log".to_string())
                            );
                            param_map.insert(
                                serde_yaml::Value::String("message".to_string()),
                                serde_yaml::Value::String("Step 1 of composite action".to_string())
                            );
                            param_map
                        })
                    );
                    map.insert(
                        serde_yaml::Value::String("outputs".to_string()),
                        serde_yaml::Value::Mapping(serde_yaml::mapping::Mapping::new())
                    );
                    map
                }),
                serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::mapping::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("type".to_string()),
                        serde_yaml::Value::String("builtin".to_string())
                    );
                    map.insert(
                        serde_yaml::Value::String("parameters".to_string()),
                        serde_yaml::Value::Mapping({
                            let mut param_map = serde_yaml::mapping::Mapping::new();
                            param_map.insert(
                                serde_yaml::Value::String("operation".to_string()),
                                serde_yaml::Value::String("sleep".to_string())
                            );
                            param_map.insert(
                                serde_yaml::Value::String("duration".to_string()),
                                serde_yaml::Value::Number(100.into())
                            );
                            param_map
                        })
                    );
                    map.insert(
                        serde_yaml::Value::String("outputs".to_string()),
                        serde_yaml::Value::Mapping(serde_yaml::mapping::Mapping::new())
                    );
                    map
                })
            ];

            params.insert("actions".to_string(), serde_yaml::Value::Sequence(sub_actions));
            params
        },
        outputs: HashMap::new(),
    };

    match EnhancedTaskExecutor::execute_action_by_type(&composite_action, context.clone()).await {
        Ok(_) => println!("✅ Composite action executed successfully"),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Print final context state
    println!("\n🎯 Final Context State:");
    println!("=======================");
    let guard = context.lock().await;
    guard.print_summary();

    println!("\n✨ Demo completed successfully!");

    Ok(())
}