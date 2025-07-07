use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;

/// 表达式求值器，用于处理工作流中的变量和表达式
#[derive(Clone)]
pub struct ExpressionEvaluator {
    env_vars: HashMap<String, String>,
    flow_vars: HashMap<String, serde_yaml::Value>,
    context_vars: HashMap<String, serde_yaml::Value>,
}

impl ExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new() -> Self {
        Self {
            env_vars: HashMap::new(),
            flow_vars: HashMap::new(),
            context_vars: HashMap::new(),
        }
    }

    /// 设置环境变量
    pub fn set_env_vars(&mut self, env_vars: HashMap<String, String>) {
        self.env_vars = env_vars;
    }

    /// 设置流程变量
    pub fn set_flow_vars(&mut self, flow_vars: HashMap<String, serde_yaml::Value>) {
        self.flow_vars = flow_vars;
    }

    /// 设置上下文变量
    pub fn set_context_var<S: AsRef<str>>(&mut self, key: S, value: serde_yaml::Value) {
        self.context_vars.insert(key.as_ref().to_string(), value);
    }

    /// 获取上下文变量
    pub fn get_context_var<S: AsRef<str>>(&self, key: S) -> Option<&serde_yaml::Value> {
        self.context_vars.get(key.as_ref())
    }

    /// 求值表达式字符串
    pub fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        // 处理环境变量引用: ${{ env.VAR_NAME }}
        let env_regex =
            Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}").context("Failed to compile env regex")?;

        let mut result = expression.to_string();

        for cap in env_regex.captures_iter(expression) {
            let var_name = &cap[1];
            if let Some(env_value) = self.env_vars.get(var_name) {
                result = result.replace(&cap[0], env_value);
            } else {
                return Err(anyhow::anyhow!(
                    "Environment variable not found: {}",
                    var_name
                ));
            }
        }

        // 处理流程变量引用: ${{ vars.VAR_NAME }}
        let vars_regex =
            Regex::new(r"\$\{\{\s*vars\.(\w+)\s*\}\}").context("Failed to compile vars regex")?;

        for cap in vars_regex.captures_iter(&result.clone()) {
            let var_name = &cap[1];
            if let Some(var_value) = self.flow_vars.get(var_name) {
                let value_str = self.yaml_value_to_string(var_value);
                result = result.replace(&cap[0], &value_str);
            } else {
                return Err(anyhow::anyhow!("Flow variable not found: {}", var_name));
            }
        }

        // 处理任务输出引用: ${task.action.outputs.field}
        let output_regex =
            Regex::new(r"\$\{([^}]+)\}").context("Failed to compile output regex")?;

        for cap in output_regex.captures_iter(&result.clone()) {
            let path = &cap[1];
            if let Some(value) = self.resolve_context_path(path)? {
                let value_str = self.yaml_value_to_string(&value);
                result = result.replace(&cap[0], &value_str);
            } else {
                return Err(anyhow::anyhow!("Context path not found: {}", path));
            }
        }

        // 尝试解析结果为合适的类型
        self.parse_result(&result)
    }

    /// 求值条件表达式，返回布尔值
    pub fn evaluate_condition(&self, condition: &str) -> Result<bool> {
        let result = self.evaluate(condition)?;

        match result {
            serde_yaml::Value::Bool(b) => Ok(b),
            serde_yaml::Value::String(s) => {
                // 简单的条件解析
                if s.contains("==") {
                    self.evaluate_equality(&s)
                } else if s.contains("!=") {
                    self.evaluate_inequality(&s)
                } else if s.contains("&&") {
                    self.evaluate_and(&s)
                } else if s.contains("||") {
                    self.evaluate_or(&s)
                } else {
                    // 非空字符串视为 true
                    Ok(!s.is_empty())
                }
            }
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(i != 0)
                } else if let Some(f) = n.as_f64() {
                    Ok(f != 0.0)
                } else {
                    Ok(false)
                }
            }
            serde_yaml::Value::Null => Ok(false),
            _ => Ok(true),
        }
    }

    fn resolve_context_path(&self, path: &str) -> Result<Option<serde_yaml::Value>> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.len() < 2 {
            return Ok(None);
        }

        // 尝试从上下文变量中解析路径
        let key = parts.join(".");
        if let Some(value) = self.context_vars.get(&key) {
            return Ok(Some(value.clone()));
        }

        // 尝试按路径结构解析
        let mut current = None;
        for (ctx_key, ctx_value) in &self.context_vars {
            if ctx_key.starts_with(parts[0]) {
                current = Some(ctx_value.clone());
                break;
            }
        }

        Ok(current)
    }

    fn yaml_value_to_string(&self, value: &serde_yaml::Value) -> String {
        match value {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Null => "null".to_string(),
            _ => serde_yaml::to_string(value).unwrap_or_default(),
        }
    }

    fn parse_result(&self, result: &str) -> Result<serde_yaml::Value> {
        // 尝试解析为数字
        if let Ok(i) = result.parse::<i64>() {
            return Ok(serde_yaml::Value::Number(serde_yaml::Number::from(i)));
        }

        if let Ok(f) = result.parse::<f64>() {
            return Ok(serde_yaml::Value::Number(serde_yaml::Number::from(f)));
        }

        // 尝试解析为布尔值
        if let Ok(b) = result.parse::<bool>() {
            return Ok(serde_yaml::Value::Bool(b));
        }

        // 默认为字符串
        Ok(serde_yaml::Value::String(result.to_string()))
    }

    fn evaluate_equality(&self, expr: &str) -> Result<bool> {
        let parts: Vec<&str> = expr.splitn(2, "==").collect();
        if parts.len() != 2 {
            return Ok(false);
        }

        let left = parts[0].trim();
        let right = parts[1].trim();
        Ok(left == right)
    }

    fn evaluate_inequality(&self, expr: &str) -> Result<bool> {
        let parts: Vec<&str> = expr.splitn(2, "!=").collect();
        if parts.len() != 2 {
            return Ok(false);
        }

        let left = parts[0].trim();
        let right = parts[1].trim();
        Ok(left != right)
    }

    fn evaluate_and(&self, expr: &str) -> Result<bool> {
        let parts: Vec<&str> = expr.split("&&").collect();
        for part in parts {
            if !self.evaluate_condition(part.trim())? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_or(&self, expr: &str) -> Result<bool> {
        let parts: Vec<&str> = expr.split("||").collect();
        for part in parts {
            if self.evaluate_condition(part.trim())? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_evaluation() {
        let mut evaluator = ExpressionEvaluator::new();
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
        evaluator.set_env_vars(env_vars);

        let result = evaluator.evaluate("${{ env.TEST_VAR }}").unwrap();
        assert_eq!(result, serde_yaml::Value::String("test_value".to_string()));
    }

    #[test]
    fn test_flow_var_evaluation() {
        let mut evaluator = ExpressionEvaluator::new();
        let mut flow_vars = HashMap::new();
        flow_vars.insert(
            "name".to_string(),
            serde_yaml::Value::String("FlowBuilder".to_string()),
        );
        evaluator.set_flow_vars(flow_vars);

        let result = evaluator.evaluate("${{ vars.name }}").unwrap();
        assert_eq!(result, serde_yaml::Value::String("FlowBuilder".to_string()));
    }

    #[test]
    fn test_condition_evaluation() {
        let evaluator = ExpressionEvaluator::new();

        assert!(evaluator.evaluate_condition("true").unwrap());
        assert!(!evaluator.evaluate_condition("false").unwrap());
        assert!(evaluator.evaluate_condition("test == test").unwrap());
        assert!(!evaluator.evaluate_condition("test != test").unwrap());
    }
}
