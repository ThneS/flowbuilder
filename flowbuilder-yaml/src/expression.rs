use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Expression provider trait for pluggable expression evaluation
pub trait ExpressionProvider {
    /// Evaluate an expression and return a YAML value
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value>;
    
    /// Get the provider name
    fn name(&self) -> &str;
}

/// Environment variable provider: ${env:VAR_NAME}
struct EnvProvider {
    env_vars: HashMap<String, String>,
}

impl ExpressionProvider for EnvProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        if let Some(value) = self.env_vars.get(expression) {
            Ok(serde_yaml::Value::String(value.clone()))
        } else {
            Err(anyhow::anyhow!("Environment variable not found: {}", expression))
        }
    }
    
    fn name(&self) -> &str {
        "env"
    }
}

/// Context provider: ${ctx:action.outputs.key}
struct ContextProvider {
    context_vars: HashMap<String, serde_yaml::Value>,
    flow_vars: HashMap<String, serde_yaml::Value>,
}

impl ExpressionProvider for ContextProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        // Handle vars.* references
        if expression.starts_with("vars.") {
            let var_name = &expression[5..]; // Remove "vars." prefix
            if let Some(value) = self.flow_vars.get(var_name) {
                return Ok(value.clone());
            } else {
                return Err(anyhow::anyhow!("Flow variable not found: {}", var_name));
            }
        }
        
        // Handle context path resolution
        if let Some(value) = self.resolve_context_path(expression)? {
            Ok(value)
        } else {
            Err(anyhow::anyhow!("Context path not found: {}", expression))
        }
    }
    
    fn name(&self) -> &str {
        "ctx"
    }
}

impl ContextProvider {
    fn resolve_context_path(&self, path: &str) -> Result<Option<serde_yaml::Value>> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return Ok(None);
        }

        // Try exact match first
        if let Some(value) = self.context_vars.get(path) {
            return Ok(Some(value.clone()));
        }

        // Try path resolution
        let mut current = None;
        for (ctx_key, ctx_value) in &self.context_vars {
            if ctx_key == path || ctx_key.starts_with(&format!("{}.", parts[0])) {
                current = Some(ctx_value.clone());
                break;
            }
        }

        Ok(current)
    }
}

/// JQ provider using xqpath: ${jq:expression}
struct JqProvider {
    context_tree: JsonValue,
    flow_vars: HashMap<String, serde_yaml::Value>,
    env_vars: HashMap<String, String>,
}

impl ExpressionProvider for JqProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        // Check for pipe syntax: setup.outputs.payload | .[0].name
        let (root_expr, jq_expr) = if expression.contains(" | ") {
            let parts: Vec<&str> = expression.splitn(2, " | ").collect();
            (Some(parts[0].trim()), parts[1].trim())
        } else {
            (None, expression)
        };
        
        // Determine the root for evaluation
        let root = if let Some(root_path) = root_expr {
            // Use specific action output as root
            self.get_root_from_path(root_path)?
        } else if expression.starts_with("vars.") {
            // Switch root to flow vars
            self.vars_to_json()?
        } else if expression.starts_with("env.") {
            // Switch root to env vars
            self.env_to_json()?
        } else {
            // Default to context tree
            self.context_tree.clone()
        };
        
        // Convert JSON to string for xqpath processing
        let root_str = serde_json::to_string(&root)?;
        
        // Evaluate the jq expression using xqpath
        let result = xqpath::query!(root_str.as_str(), jq_expr)
            .map_err(|e| anyhow::anyhow!("JQ evaluation error: {}", e))?;
            
        // Convert result back to YAML value
        self.json_to_yaml(result)
    }
    
    fn name(&self) -> &str {
        "jq"
    }
}

impl JqProvider {
    fn get_root_from_path(&self, path: &str) -> Result<JsonValue> {
        // Navigate to the specified path in context tree
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.context_tree;
        
        for part in parts {
            if let Some(value) = current.get(part) {
                current = value;
            } else {
                return Err(anyhow::anyhow!("Path not found in context: {}", path));
            }
        }
        
        Ok(current.clone())
    }
    
    fn vars_to_json(&self) -> Result<JsonValue> {
        let mut vars_map = serde_json::Map::new();
        for (key, value) in &self.flow_vars {
            let json_value = self.yaml_to_json(value.clone())?;
            vars_map.insert(key.clone(), json_value);
        }
        Ok(JsonValue::Object(vars_map))
    }
    
    fn env_to_json(&self) -> Result<JsonValue> {
        let mut env_map = serde_json::Map::new();
        for (key, value) in &self.env_vars {
            env_map.insert(key.clone(), JsonValue::String(value.clone()));
        }
        Ok(JsonValue::Object(env_map))
    }
    
    fn yaml_to_json(&self, value: serde_yaml::Value) -> Result<JsonValue> {
        let json_str = serde_json::to_string(&value)?;
        let json_value: JsonValue = serde_json::from_str(&json_str)?;
        Ok(json_value)
    }
    
    fn json_to_yaml(&self, results: Vec<JsonValue>) -> Result<serde_yaml::Value> {
        // If single result, return it directly; if multiple, return as array
        let json_result = if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            JsonValue::Array(results)
        };
        
        let yaml_str = serde_yaml::to_string(&json_result)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;
        Ok(yaml_value)
    }
}

/// Token representing a parsed expression part
#[derive(Debug, Clone)]
enum ExpressionToken {
    /// Literal text that should be preserved as-is
    Literal(String),
    /// Provider-based expression: ${provider:expression}
    ProviderExpression { provider: String, expression: String },
    /// Legacy expression that needs mapping to provider
    LegacyExpression(String),
}

/// Expression tokenizer and normalizer
struct ExpressionTokenizer;

impl ExpressionTokenizer {
    /// Parse an expression string into tokens
    fn tokenize(input: &str) -> Result<Vec<ExpressionToken>> {
        let mut tokens = Vec::new();
        let mut current_pos = 0;
        
        // Regex patterns for different expression types
        let provider_regex = Regex::new(r"\$\{(\w+):([^}]+)\}")
            .context("Failed to compile provider regex")?;
        let legacy_env_regex = Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}")
            .context("Failed to compile legacy env regex")?;
        let legacy_vars_regex = Regex::new(r"\$\{\{\s*vars\.(\w+)\s*\}\}")
            .context("Failed to compile legacy vars regex")?;
        let legacy_output_regex = Regex::new(r"\$\{([^}:]+)\}")
            .context("Failed to compile legacy output regex")?;
        let legacy_env_simple_regex = Regex::new(r"\$env:(\w+)")
            .context("Failed to compile legacy env simple regex")?;
        let legacy_jq_regex = Regex::new(r"\$jq:([^$\s]+)")
            .context("Failed to compile legacy jq regex")?;
        
        let mut matches = Vec::new();
        
        // Collect all matches with their positions
        for cap in provider_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                    provider: cap[1].to_string(),
                    expression: cap[2].to_string(),
                }));
            }
        }
        
        for cap in legacy_env_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                    provider: "env".to_string(),
                    expression: cap[1].to_string(),
                }));
            }
        }
        
        for cap in legacy_vars_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                    provider: "ctx".to_string(),
                    expression: format!("vars.{}", &cap[1]),
                }));
            }
        }
        
        for cap in legacy_env_simple_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                    provider: "env".to_string(),
                    expression: cap[1].to_string(),
                }));
            }
        }
        
        for cap in legacy_jq_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                    provider: "jq".to_string(),
                    expression: cap[1].to_string(),
                }));
            }
        }
        
        for cap in legacy_output_regex.captures_iter(input) {
            if let Some(mat) = cap.get(0) {
                // Only match if it's not already matched by provider regex
                let already_matched = matches.iter().any(|(start, end, _)| {
                    mat.start() >= *start && mat.end() <= *end
                });
                if !already_matched {
                    matches.push((mat.start(), mat.end(), ExpressionToken::ProviderExpression {
                        provider: "ctx".to_string(),
                        expression: cap[1].to_string(),
                    }));
                }
            }
        }
        
        // Sort matches by position
        matches.sort_by_key(|(start, _, _)| *start);
        
        // Build tokens with literals in between
        for (start, end, token) in matches {
            // Add literal text before this match
            if current_pos < start {
                let literal = input[current_pos..start].to_string();
                if !literal.is_empty() {
                    tokens.push(ExpressionToken::Literal(literal));
                }
            }
            
            tokens.push(token);
            current_pos = end;
        }
        
        // Add remaining literal text
        if current_pos < input.len() {
            let literal = input[current_pos..].to_string();
            if !literal.is_empty() {
                tokens.push(ExpressionToken::Literal(literal));
            }
        }
        
        // If no expressions found, treat the whole string as literal
        if tokens.is_empty() && !input.is_empty() {
            tokens.push(ExpressionToken::Literal(input.to_string()));
        }
        
        Ok(tokens)
    }
}

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
    pub fn set_flow_vars(
        &mut self,
        flow_vars: HashMap<String, serde_yaml::Value>,
    ) {
        self.flow_vars = flow_vars;
    }

    /// 设置上下文变量
    pub fn set_context_var<S: AsRef<str>>(
        &mut self,
        key: S,
        value: serde_yaml::Value,
    ) {
        self.context_vars.insert(key.as_ref().to_string(), value);
    }

    /// 获取上下文变量
    pub fn get_context_var<S: AsRef<str>>(
        &self,
        key: S,
    ) -> Option<&serde_yaml::Value> {
        self.context_vars.get(key.as_ref())
    }

    /// Evaluate a YAML value, preserving types for single expressions or doing string interpolation for mixed content
    pub fn evaluate_yaml(&self, value: &serde_yaml::Value) -> Result<serde_yaml::Value> {
        match value {
            serde_yaml::Value::String(s) => {
                let tokens = ExpressionTokenizer::tokenize(s)?;
                
                // If it's a single provider expression, return the native type
                if tokens.len() == 1 {
                    if let ExpressionToken::ProviderExpression { provider, expression } = &tokens[0] {
                        return self.evaluate_provider_expression(provider, expression);
                    }
                }
                
                // Otherwise, do string interpolation
                let mut result = String::new();
                for token in tokens {
                    match token {
                        ExpressionToken::Literal(text) => result.push_str(&text),
                        ExpressionToken::ProviderExpression { provider, expression } => {
                            let value = self.evaluate_provider_expression(&provider, &expression)?;
                            result.push_str(&self.yaml_value_to_string(&value));
                        }
                        ExpressionToken::LegacyExpression(expr) => {
                            // This shouldn't happen with current tokenizer, but handle just in case
                            let value = self.evaluate(&expr)?;
                            result.push_str(&self.yaml_value_to_string(&value));
                        }
                    }
                }
                
                // Parse the result to the appropriate type
                self.parse_result(&result)
            }
            // For non-string values, return as-is
            _ => Ok(value.clone()),
        }
    }

    /// Create a context tree for jq provider
    fn build_context_tree(&self) -> Result<JsonValue> {
        let mut context_map = serde_json::Map::new();
        
        // Add all context variables to the tree
        for (key, value) in &self.context_vars {
            let json_value = self.yaml_to_json(value.clone())?;
            
            // Parse nested keys like "action.outputs.key"
            let parts: Vec<&str> = key.split('.').collect();
            
            if parts.len() == 1 {
                // Simple key
                context_map.insert(key.clone(), json_value);
            } else {
                // Nested key - build the nested structure
                let mut nested_map = serde_json::Map::new();
                nested_map.insert(parts[parts.len() - 1].to_string(), json_value);
                
                // Build from the inside out
                for i in (0..parts.len() - 1).rev() {
                    let mut outer_map = serde_json::Map::new();
                    if i == parts.len() - 2 {
                        outer_map.insert(parts[i + 1].to_string(), JsonValue::Object(nested_map));
                    } else {
                        outer_map.insert(parts[i + 1].to_string(), JsonValue::Object(nested_map));
                    }
                    nested_map = outer_map;
                }
                
                // Insert the top-level key
                let top_key = parts[0].to_string();
                if context_map.contains_key(&top_key) {
                    // Merge with existing nested structure
                    if let Some(JsonValue::Object(existing)) = context_map.get_mut(&top_key) {
                        if let JsonValue::Object(new_nested) = JsonValue::Object(nested_map) {
                            for (k, v) in new_nested {
                                existing.insert(k, v);
                            }
                        }
                    }
                } else {
                    context_map.insert(top_key, JsonValue::Object(nested_map));
                }
            }
        }
        
        Ok(JsonValue::Object(context_map))
    }

    /// Evaluate a provider expression
    fn evaluate_provider_expression(&self, provider: &str, expression: &str) -> Result<serde_yaml::Value> {
        match provider {
            "env" => {
                let env_provider = EnvProvider {
                    env_vars: self.env_vars.clone(),
                };
                env_provider.evaluate(expression)
            }
            "ctx" => {
                let ctx_provider = ContextProvider {
                    context_vars: self.context_vars.clone(),
                    flow_vars: self.flow_vars.clone(),
                };
                ctx_provider.evaluate(expression)
            }
            "jq" => {
                let context_tree = self.build_context_tree()?;
                let jq_provider = JqProvider {
                    context_tree,
                    flow_vars: self.flow_vars.clone(),
                    env_vars: self.env_vars.clone(),
                };
                jq_provider.evaluate(expression)
            }
            _ => Err(anyhow::anyhow!("Unknown provider: {}", provider))
        }
    }

    /// 求值表达式字符串 (legacy method for backward compatibility)
    pub fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        let tokens = ExpressionTokenizer::tokenize(expression)?;
        
        // If it's a single provider expression, return the native type
        if tokens.len() == 1 {
            if let ExpressionToken::ProviderExpression { provider, expression } = &tokens[0] {
                return self.evaluate_provider_expression(provider, expression);
            }
        }
        
        // Otherwise, do string interpolation
        let mut result = String::new();
        for token in tokens {
            match token {
                ExpressionToken::Literal(text) => result.push_str(&text),
                ExpressionToken::ProviderExpression { provider, expression } => {
                    let value = self.evaluate_provider_expression(&provider, &expression)?;
                    result.push_str(&self.yaml_value_to_string(&value));
                }
                ExpressionToken::LegacyExpression(expr) => {
                    // Fallback to legacy evaluation
                    let value = self.legacy_evaluate(&expr)?;
                    result.push_str(&self.yaml_value_to_string(&value));
                }
            }
        }
        
        // Parse the result to the appropriate type
        self.parse_result(&result)
    }

    /// Legacy evaluation method (for any expressions that don't match new patterns)
    fn legacy_evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        // Handle legacy patterns that might not be caught by tokenizer
        
        // 处理环境变量引用: ${{ env.VAR_NAME }}
        let env_regex = Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}")
            .context("Failed to compile env regex")?;

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
        let vars_regex = Regex::new(r"\$\{\{\s*vars\.(\w+)\s*\}\}")
            .context("Failed to compile vars regex")?;

        for cap in vars_regex.captures_iter(&result.clone()) {
            let var_name = &cap[1];
            if let Some(var_value) = self.flow_vars.get(var_name) {
                let value_str = self.yaml_value_to_string(var_value);
                result = result.replace(&cap[0], &value_str);
            } else {
                return Err(anyhow::anyhow!(
                    "Flow variable not found: {}",
                    var_name
                ));
            }
        }

        // 处理任务输出引用: ${task.action.outputs.field}
        let output_regex = Regex::new(r"\$\{([^}]+)\}")
            .context("Failed to compile output regex")?;

        for cap in output_regex.captures_iter(&result.clone()) {
            let path = &cap[1];
            if let Some(value) = self.resolve_context_path(path)? {
                let value_str = self.yaml_value_to_string(&value);
                result = result.replace(&cap[0], &value_str);
            } else {
                return Err(anyhow::anyhow!(
                    "Context path not found: {}",
                    path
                ));
            }
        }

        // 尝试解析结果为合适的类型
        self.parse_result(&result)
    }

    /// Helper to convert YAML to JSON
    fn yaml_to_json(&self, value: serde_yaml::Value) -> Result<JsonValue> {
        let json_str = serde_json::to_string(&value)?;
        let json_value: JsonValue = serde_json::from_str(&json_str)?;
        Ok(json_value)
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

    fn resolve_context_path(
        &self,
        path: &str,
    ) -> Result<Option<serde_yaml::Value>> {
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
        assert_eq!(
            result,
            serde_yaml::Value::String("FlowBuilder".to_string())
        );
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
