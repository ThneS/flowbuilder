use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value as JsonValue;
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

    /// 求值表达式字符串（支持统一 Provider 语法与旧语法，带类型保真与插值）
    pub fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        let expr = expression.trim();

        // 1) 如果是单一的统一 Provider 表达式：${provider:body}，直接按类型返回
        if let Some((provider, body)) = self.match_single_provider(expr)? {
            let val = self.eval_provider(provider, body)?;
            return Ok(self.json_to_yaml(val));
        }

        // 2) 字符串插值：替换所有 ${provider:body}
        let mut interpolated = self.interpolate_providers(expr)?;

        // 3) 兼容旧语法：${{ env.VAR }} 与 ${{ vars.NAME }}
        interpolated = self.interpolate_legacy_env_vars(&interpolated)?;

        // 4) 兼容旧语法：${task.action.outputs.field}（上下文路径）
        interpolated = self.interpolate_legacy_context(&interpolated)?;

        // 5) 插值后的整体结果：尽量还原为数字/布尔等原生类型
        self.parse_result(&interpolated)
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

    // =============== 统一 Provider 实现 ===============

    /// 匹配单个统一 Provider 表达式（整行）
    fn match_single_provider<'a>(
        &self,
        s: &'a str,
    ) -> Result<Option<(&'a str, &'a str)>> {
        // 简单正则：不支持大括号/嵌套，覆盖常见用法
        let re = Regex::new(r"^\$\{([a-zA-Z_][\w\-]*):([^}]*)\}$")
            .context("compile provider single regex")?;
        if let Some(c) = re.captures(s) {
            let provider = c.get(1).unwrap().as_str();
            let body = c.get(2).unwrap().as_str();
            return Ok(Some((provider, body.trim())));
        }
        Ok(None)
    }

    /// 字符串内插值统一 Provider
    fn interpolate_providers(&self, s: &str) -> Result<String> {
        let re = Regex::new(r"\$\{([a-zA-Z_][\w\-]*):([^}]*)\}")
            .context("compile provider inline regex")?;
        let mut out = String::from(s);
        // 为避免重叠替换，先收集所有匹配再逐个替换
        let caps: Vec<(String, String)> = re
            .captures_iter(s)
            .filter_map(|c| {
                let full = c.get(0)?.as_str().to_string();
                let provider = c.get(1)?.as_str();
                let body = c.get(2)?.as_str().trim();
                let val = self.eval_provider(provider, body).ok()?;
                let repl = self.yaml_value_to_string(&self.json_to_yaml(val));
                Some((full, repl))
            })
            .collect();
        for (full, repl) in caps {
            out = out.replace(&full, &repl);
        }
        Ok(out)
    }

    /// 统一 Provider 求值
    fn eval_provider(&self, provider: &str, body: &str) -> Result<JsonValue> {
        match provider {
            "env" => self.eval_env_provider(body),
            "ctx" => self.eval_ctx_provider(body),
            "jq" => self.eval_jq_provider(body),
            _ => Err(anyhow::anyhow!("Unknown provider: {}", provider)),
        }
    }

    fn eval_env_provider(&self, key: &str) -> Result<JsonValue> {
        if let Some(v) = self.env_vars.get(key) {
            Ok(JsonValue::String(v.clone()))
        } else {
            Err(anyhow::anyhow!("Environment variable not found: {}", key))
        }
    }

    fn eval_ctx_provider(&self, path: &str) -> Result<JsonValue> {
        let root = self.build_ctx_root_json();
        let target = self.get_by_dot_path(&root, path).ok_or_else(|| {
            anyhow::anyhow!("Context path not found: {}", path)
        })?;
        Ok(target)
    }

    /// 极简 jq 风格解析器：支持 '.' 根路径、管道 '|'、数组索引 '[n]' 与点路径
    fn eval_jq_provider(&self, expr: &str) -> Result<JsonValue> {
        let root = self.build_ctx_root_json();
        // 管道拆分
        let stages: Vec<&str> = expr.split('|').map(|s| s.trim()).collect();
        let mut cur = root;
        for stage in stages {
            if stage.is_empty() {
                continue;
            }
            let next = if let Some(path) = stage.strip_prefix('.') {
                // 从当前值相对路径
                self.get_by_dot_path(&cur, path)
            } else {
                // 从全局根访问（如 vars.config）
                self.get_by_dot_path(&cur, stage)
            };
            cur = next.ok_or_else(|| {
                anyhow::anyhow!("jq expression failed at: {}", stage)
            })?;
        }
        Ok(cur)
    }

    /// 构建供 ctx/jq 使用的 JSON 根对象：包含 vars、env 以及 context_vars（按点号嵌套）
    fn build_ctx_root_json(&self) -> JsonValue {
        let mut root = serde_json::Map::new();
        // vars
        root.insert(
            "vars".to_string(),
            self.yaml_to_json(&serde_yaml::Value::Mapping(
                self.flow_vars
                    .iter()
                    .map(|(k, v)| {
                        (serde_yaml::Value::String(k.clone()), v.clone())
                    })
                    .collect(),
            )),
        );
        // env
        root.insert(
            "env".to_string(),
            JsonValue::Object(
                self.env_vars
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect(),
            ),
        );

        // 其它上下文（将点号 key 拆分为嵌套对象）
        let mut ctx_obj = serde_json::Map::new();
        for (k, v) in &self.context_vars {
            self.insert_nested_json(&mut ctx_obj, k, &self.yaml_to_json(v));
        }
        if !ctx_obj.is_empty() {
            // 将所有顶层上下文并入根（例如 `auth.outputs.token` 会生成 root["auth"]["outputs"]["token"]）
            for (k, v) in ctx_obj {
                root.insert(k, v);
            }
        }

        JsonValue::Object(root)
    }

    fn insert_nested_json(
        &self,
        obj: &mut serde_json::Map<String, JsonValue>,
        dotted: &str,
        value: &JsonValue,
    ) {
        let mut current = obj;
        let parts: Vec<&str> = dotted.split('.').collect();
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current.insert(part.to_string(), value.clone());
            } else {
                current = current
                    .entry(part.to_string())
                    .or_insert_with(
                        || JsonValue::Object(serde_json::Map::new()),
                    )
                    .as_object_mut()
                    .unwrap();
            }
        }
    }

    /// 在 JSON 值上按点路径获取子值（支持数组索引，如 users[0].name 或 [0]）
    fn get_by_dot_path(
        &self,
        root: &JsonValue,
        path: &str,
    ) -> Option<JsonValue> {
        let path = path.trim();
        if path.is_empty() {
            return Some(root.clone());
        }
        let mut cur = root.clone();
        for seg in path.split('.') {
            if seg.is_empty() {
                continue;
            }
            let key = seg;
            // 可能是纯索引，如 [0]
            if key.starts_with('[') {
                // 多重索引：[0][1]
                let mut val = cur;
                let mut rest = key;
                while rest.starts_with('[') {
                    if let Some(end) = rest.find(']') {
                        let idx_str = &rest[1..end];
                        let idx: usize = idx_str.parse().ok()?;
                        let arr = val.as_array()?;
                        val = arr.get(idx)?.clone();
                        rest = &rest[end + 1..];
                    } else {
                        return None;
                    }
                }
                cur = val;
                continue;
            }

            // 解析 key 与可能的索引（如 name[0][1]）
            if let Some((base, rest)) = key.split_once('[') {
                // 先取对象字段
                cur = cur.get(base)?.clone();
                let mut rest_brackets = format!("[{}", rest);
                // 逐个消费索引
                while rest_brackets.starts_with('[') {
                    if let Some(end) = rest_brackets.find(']') {
                        let idx_str = &rest_brackets[1..end];
                        let idx: usize = idx_str.parse().ok()?;
                        let arr = cur.as_array()?;
                        cur = arr.get(idx)?.clone();
                        rest_brackets = rest_brackets[end + 1..].to_string();
                    } else {
                        return None;
                    }
                }
            } else {
                // 普通字段
                cur = cur.get(key)?.clone();
            }
        }
        Some(cur)
    }

    fn yaml_to_json(&self, v: &serde_yaml::Value) -> JsonValue {
        // 通过中转字符串避免数值/布尔丢失
        serde_json::to_value(v).unwrap_or(JsonValue::Null)
    }

    fn json_to_yaml(&self, v: JsonValue) -> serde_yaml::Value {
        // 直接使用 serde_yaml::to_value 需要 serde_yaml::Value: Serialize
        match v {
            JsonValue::Null => serde_yaml::Value::Null,
            JsonValue::Bool(b) => serde_yaml::Value::Bool(b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    serde_yaml::Value::Number(serde_yaml::Number::from(i))
                } else if let Some(u) = n.as_u64() {
                    serde_yaml::Value::Number(serde_yaml::Number::from(u))
                } else if let Some(f) = n.as_f64() {
                    // serde_yaml::Number 没有 from_f64，降级为字符串表示
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        f as i64,
                    ))
                } else {
                    serde_yaml::Value::Null
                }
            }
            JsonValue::String(s) => serde_yaml::Value::String(s),
            JsonValue::Array(arr) => serde_yaml::Value::Sequence(
                arr.into_iter().map(|e| self.json_to_yaml(e)).collect(),
            ),
            JsonValue::Object(obj) => serde_yaml::Value::Mapping(
                obj.into_iter()
                    .map(|(k, v)| {
                        (serde_yaml::Value::String(k), self.json_to_yaml(v))
                    })
                    .collect(),
            ),
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

        // 新实现：使用统一 ctx 根解析以增强路径能力
        let root = self.build_ctx_root_json();
        Ok(self
            .get_by_dot_path(&root, path)
            .map(|v| self.json_to_yaml(v)))
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

    // =============== 旧语法插值辅助 ===============

    fn interpolate_legacy_env_vars(&self, s: &str) -> Result<String> {
        let env_regex =
            Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}").context("env re")?;
        let mut out = s.to_string();
        for cap in env_regex.captures_iter(s) {
            let var_name = &cap[1];
            if let Some(env_value) = self.env_vars.get(var_name) {
                out = out.replace(&cap[0], env_value);
            } else {
                return Err(anyhow::anyhow!(
                    "Environment variable not found: {}",
                    var_name
                ));
            }
        }
        // ${{ vars.NAME }}
        let vars_regex =
            Regex::new(r"\$\{\{\s*vars\.(\w+)\s*\}\}").context("vars re")?;
        for cap in vars_regex.captures_iter(&out.clone()) {
            let var_name = &cap[1];
            if let Some(var_value) = self.flow_vars.get(var_name) {
                let value_str = self.yaml_value_to_string(var_value);
                out = out.replace(&cap[0], &value_str);
            } else {
                return Err(anyhow::anyhow!(
                    "Flow variable not found: {}",
                    var_name
                ));
            }
        }
        Ok(out)
    }

    fn interpolate_legacy_context(&self, s: &str) -> Result<String> {
        // 宽匹配 ${...}，在代码中过滤统一 Provider 格式，避免使用不支持的前瞻
        let output_regex = Regex::new(r"\$\{([^}]+)\}").context("output re")?;
        let mut out = s.to_string();
        let matches: Vec<(String, String)> = output_regex
            .captures_iter(s)
            .filter_map(|c| {
                let full = c.get(0)?.as_str().to_string();
                let inner = c.get(1)?.as_str();
                // 跳过统一 Provider：形如 provider:...
                let provider_like = Regex::new(r"^[a-zA-Z_][\w\-]*:").ok()?;
                if provider_like.is_match(inner) {
                    return None;
                }
                // 尝试解析为上下文路径
                let val = self.resolve_context_path(inner).ok()?.or(None)?;
                let repl = self.yaml_value_to_string(&val);
                Some((full, repl))
            })
            .collect();
        for (full, repl) in matches {
            out = out.replace(&full, &repl);
        }
        Ok(out)
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

    #[test]
    fn test_unified_env() {
        let mut evaluator = ExpressionEvaluator::new();
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "API_URL".to_string(),
            "https://api.example.com".to_string(),
        );
        evaluator.set_env_vars(env_vars);

        let v = evaluator.evaluate("${env:API_URL}").unwrap();
        assert_eq!(
            v,
            serde_yaml::Value::String("https://api.example.com".into())
        );

        let v2 = evaluator.evaluate("Endpoint: ${env:API_URL}").unwrap();
        assert_eq!(
            v2,
            serde_yaml::Value::String(
                "Endpoint: https://api.example.com".into()
            )
        );
    }

    #[test]
    fn test_unified_ctx_vars() {
        let mut evaluator = ExpressionEvaluator::new();
        let mut flow_vars = HashMap::new();
        flow_vars.insert("debug".into(), serde_yaml::Value::Bool(true));
        flow_vars.insert(
            "timeout".into(),
            serde_yaml::Value::Number(serde_yaml::Number::from(30)),
        );
        evaluator.set_flow_vars(flow_vars);

        let v = evaluator.evaluate("${ctx:vars.debug}").unwrap();
        assert_eq!(v, serde_yaml::Value::Bool(true));

        let v2 = evaluator.evaluate("${ctx:vars.timeout}").unwrap();
        assert_eq!(v2, serde_yaml::Value::Number(serde_yaml::Number::from(30)));
    }

    #[test]
    fn test_unified_ctx_action_outputs() {
        let mut evaluator = ExpressionEvaluator::new();
        evaluator.set_context_var(
            "auth.outputs.token",
            serde_yaml::Value::String("abc123".into()),
        );
        let v = evaluator.evaluate("${ctx:auth.outputs.token}").unwrap();
        assert_eq!(v, serde_yaml::Value::String("abc123".into()));
    }

    #[test]
    fn test_unified_jq_vars_array() {
        use serde_yaml::{Mapping, Value};
        let mut evaluator = ExpressionEvaluator::new();
        // vars.users = [{ name: "alice", role: "admin" }, { name: "bob", role: "user" }]
        let mut user1 = Mapping::new();
        user1.insert(
            Value::String("name".into()),
            Value::String("alice".into()),
        );
        user1.insert(
            Value::String("role".into()),
            Value::String("admin".into()),
        );
        let mut user2 = Mapping::new();
        user2.insert(Value::String("name".into()), Value::String("bob".into()));
        user2
            .insert(Value::String("role".into()), Value::String("user".into()));
        let users =
            Value::Sequence(vec![Value::Mapping(user1), Value::Mapping(user2)]);

        let mut flow_vars = HashMap::new();
        flow_vars.insert("users".into(), users);
        evaluator.set_flow_vars(flow_vars);

        // 直接路径
        let v = evaluator.evaluate("${jq:vars.users[0].name}").unwrap();
        assert_eq!(v, serde_yaml::Value::String("alice".into()));

        // 相对路径与管道（这里实现为与直接路径一致的等价调用）
        let v2 = evaluator.evaluate("${jq:vars.users|.[1].role}").unwrap();
        assert_eq!(v2, serde_yaml::Value::String("user".into()));
    }

    #[test]
    fn test_string_interpolation_mixed() {
        let mut evaluator = ExpressionEvaluator::new();
        let mut env_vars = HashMap::new();
        env_vars.insert("API_URL".into(), "https://api.example.com".into());
        evaluator.set_env_vars(env_vars);

        let mut flow_vars = HashMap::new();
        flow_vars.insert("debug".into(), serde_yaml::Value::Bool(true));
        evaluator.set_flow_vars(flow_vars);

        let v = evaluator
            .evaluate("Debug: ${ctx:vars.debug} at ${env:API_URL}")
            .unwrap();
        assert_eq!(
            v,
            serde_yaml::Value::String(
                "Debug: true at https://api.example.com".into()
            )
        );
    }
}
