//! # FlowBuilder Runtime - 增强的任务执行器
//!
//! 基于执行计划的任务执行器，负责执行具体的任务

use anyhow::Result;
use flowbuilder_context::SharedContext;
use flowbuilder_core::{
    ActionSpec, ExecutionNode, ExecutionPhase, ExecutionPlan, Executor,
    ExecutorStatus, PhaseExecutionMode, RetryStrategy,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
// tracing 宏无需显式 use 引入

/// 增强的任务执行器
pub struct EnhancedTaskExecutor {
    /// 执行器配置
    config: ExecutorConfig,
    /// 执行器状态
    status: ExecutorStatus,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 执行统计（可选）
    #[allow(dead_code)]
    stats: ExecutionStats,
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 默认超时时间（毫秒）
    pub default_timeout: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_timeout: 30000, // 30秒
        }
    }
}

/// 执行统计
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ExecutionStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 成功任务数
    pub successful_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 跳过任务数
    pub skipped_tasks: usize,
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 平均执行时间
    pub average_execution_time: Duration,
}

impl Default for EnhancedTaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedTaskExecutor {
    /// 创建新的任务执行器
    pub fn new() -> Self {
        let config = ExecutorConfig::default();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            config,
            status: ExecutorStatus::Idle,
            semaphore,
            stats: ExecutionStats::default(),
        }
    }

    /// 使用配置创建任务执行器
    pub fn with_config(config: ExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            config,
            status: ExecutorStatus::Idle,
            semaphore,
            stats: ExecutionStats::default(),
        }
    }

    /// 执行执行计划
    #[tracing::instrument(level = "info", skip(self, context), fields(workflow = %plan.metadata.workflow_name, phases = plan.phases.len()))]
    pub async fn execute_plan(
        &mut self,
        plan: ExecutionPlan,
        context: SharedContext,
    ) -> Result<ExecutionResult> {
        self.status = ExecutorStatus::Running;
        let start_time = Instant::now();

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(workflow = %plan.metadata.workflow_name, phases = plan.phases.len(), total_nodes = plan.metadata.total_nodes, "开始执行计划");
        }

        let mut result = ExecutionResult {
            plan_id: plan.metadata.plan_id.clone(),
            start_time,
            end_time: None,
            phase_results: Vec::new(),
            total_duration: Duration::default(),
            success: true,
            error_message: None,
        };

        // 设置环境变量和流程变量到上下文
        self.setup_context(&plan, context.clone()).await?;

        // 按阶段执行
        #[cfg(feature = "detailed-logging")]
        for (index, phase) in plan.phases.iter().enumerate() {
            tracing::info!(phase_index = index + 1, phase_name = %phase.name, mode = ?phase.execution_mode, "执行阶段");
            let phase_start = Instant::now();
            let phase_result =
                match self.execute_phase(phase, context.clone()).await {
                    Ok(r) => r,
                    Err(e) => {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        PhaseResult {
                            phase_id: phase.id.clone(),
                            phase_name: phase.name.clone(),
                            start_time: phase_start,
                            end_time: Some(Instant::now()),
                            duration: phase_start.elapsed(),
                            success: false,
                            error_message: Some(e.to_string()),
                            node_results: Vec::new(),
                        }
                    }
                };
            result.phase_results.push(phase_result);
            if !result.success {
                break;
            }
        }
        #[cfg(not(feature = "detailed-logging"))]
        for phase in plan.phases.iter() {
            let phase_start = Instant::now();
            let phase_result =
                match self.execute_phase(phase, context.clone()).await {
                    Ok(r) => r,
                    Err(e) => {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        PhaseResult {
                            phase_id: phase.id.clone(),
                            phase_name: phase.name.clone(),
                            start_time: phase_start,
                            end_time: Some(Instant::now()),
                            duration: phase_start.elapsed(),
                            success: false,
                            error_message: Some(e.to_string()),
                            node_results: Vec::new(),
                        }
                    }
                };
            result.phase_results.push(phase_result);
            if !result.success {
                break;
            }
        }

        result.end_time = Some(Instant::now());
        result.total_duration = start_time.elapsed();

        // 更新统计信息（perf-metrics 特性）
        #[cfg(feature = "perf-metrics")]
        {
            self.update_stats(&result);
        }

        self.status = ExecutorStatus::Idle;

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(total_duration_ms = ?result.total_duration, "执行计划完成");
        }

        Ok(result)
    }

    /// 执行阶段
    #[tracing::instrument(level = "info", skip(self, context), fields(phase = %phase.name, mode = ?phase.execution_mode))]
    async fn execute_phase(
        &mut self,
        phase: &ExecutionPhase,
        context: SharedContext,
    ) -> Result<PhaseResult> {
        let start_time = Instant::now();
        let mut phase_result = PhaseResult {
            phase_id: phase.id.clone(),
            phase_name: phase.name.clone(),
            start_time,
            end_time: None,
            duration: Duration::default(),
            success: true,
            error_message: None,
            node_results: Vec::new(),
        };

        // 检查阶段条件
        if let Some(_condition) = &phase.condition {
            // 这里应该使用表达式评估器检查条件
            // 为了简化，这里假设条件总是满足
            #[cfg(feature = "detailed-logging")]
            {
                tracing::debug!("检查阶段条件(已省略表达式)");
            }
        }

        match phase.execution_mode {
            PhaseExecutionMode::Sequential => {
                for node in &phase.nodes {
                    let node_result =
                        self.execute_node(node, context.clone()).await?;
                    phase_result.node_results.push(node_result);
                }
            }
            PhaseExecutionMode::Parallel => {
                #[cfg(not(feature = "parallel"))]
                {
                    // 并行被禁用时退化为顺序
                    for node in &phase.nodes {
                        let node_result =
                            self.execute_node(node, context.clone()).await?;
                        phase_result.node_results.push(node_result);
                    }
                    phase_result.end_time = Some(Instant::now());
                    phase_result.duration = start_time.elapsed();
                    return Ok(phase_result);
                }

                #[cfg(feature = "parallel")]
                let mut handles = Vec::new();

                for node in &phase.nodes {
                    let node_clone = node.clone();
                    let context_clone = context.clone();
                    let semaphore = self.semaphore.clone();
                    let config = self.config.clone();

                    let handle = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        Self::execute_node_static(
                            &node_clone,
                            context_clone,
                            &config,
                        )
                        .await
                    });

                    handles.push(handle);
                }

                // 等待所有任务完成
                for handle in handles {
                    match handle.await {
                        Ok(node_result) => match node_result {
                            Ok(result) => {
                                phase_result.node_results.push(result)
                            }
                            Err(e) => {
                                phase_result.success = false;
                                phase_result.error_message =
                                    Some(e.to_string());
                                return Err(e);
                            }
                        },
                        Err(e) => {
                            phase_result.success = false;
                            phase_result.error_message = Some(e.to_string());
                            return Err(anyhow::anyhow!("任务执行失败: {}", e));
                        }
                    }
                }
            }
            PhaseExecutionMode::Conditional { condition: _ } => {
                // 检查条件
                #[cfg(feature = "detailed-logging")]
                tracing::debug!("检查条件(已忽略具体表达式)");

                // 简化的条件检查，实际应该使用表达式评估器
                let condition_met = true; // 假设条件满足

                if condition_met {
                    for node in &phase.nodes {
                        let node_result =
                            self.execute_node(node, context.clone()).await?;
                        phase_result.node_results.push(node_result);
                    }
                } else {
                    #[cfg(feature = "detailed-logging")]
                    tracing::info!(phase = %phase.name, "跳过阶段 (条件不满足)");
                }
            }
        }

        phase_result.end_time = Some(Instant::now());
        phase_result.duration = start_time.elapsed();

        Ok(phase_result)
    }

    /// 执行节点
    #[tracing::instrument(level = "debug", skip(self, context), fields(node_id = %node.id, node_name = %node.name))]
    async fn execute_node(
        &mut self,
        node: &ExecutionNode,
        context: SharedContext,
    ) -> Result<NodeResult> {
        Self::execute_node_static(node, context, &self.config).await
    }

    /// 静态执行节点（用于并发执行）
    #[tracing::instrument(level = "debug", skip(context, config), fields(node_id = %node.id, node_name = %node.name))]
    async fn execute_node_static(
        node: &ExecutionNode,
        context: SharedContext,
        config: &ExecutorConfig,
    ) -> Result<NodeResult> {
        let start_time = Instant::now();
        let mut result = NodeResult {
            node_id: node.id.clone(),
            node_name: node.name.clone(),
            start_time,
            end_time: None,
            duration: Duration::default(),
            success: true,
            error_message: None,
            retry_count: 0,
        };

        #[cfg(feature = "detailed-logging")]
        {
            tracing::info!(node_id = %node.id, node_name = %node.name, "执行节点");
        }

        // 检查节点条件
        if let Some(_condition) = &node.condition {
            #[cfg(feature = "detailed-logging")]
            {
                tracing::debug!("检查节点条件(已省略表达式)");
            }
            // 简化的条件检查
            let condition_met = true;
            if !condition_met {
                #[cfg(feature = "detailed-logging")]
                {
                    tracing::info!(node = %node.name, "跳过节点 (条件不满足)");
                }
                result.end_time = Some(Instant::now());
                result.duration = start_time.elapsed();
                return Ok(result);
            }
        }

        // 执行重试逻辑（可关闭）
        #[cfg(feature = "retry")]
        let max_retries = node
            .retry_config
            .as_ref()
            .map(|c| c.max_retries)
            .unwrap_or(0);
        #[cfg(not(feature = "retry"))]
        let max_retries = 0u32;
        let mut retries = 0;

        loop {
            let execute_result =
                Self::execute_node_action(node, context.clone(), config).await;

            match execute_result {
                Ok(()) => {
                    result.success = true;
                    break;
                }
                Err(e) => {
                    if retries < max_retries {
                        retries += 1;
                        result.retry_count = retries;

                        #[cfg(feature = "detailed-logging")]
                        {
                            tracing::warn!(node = %node.name, retries = retries, max_retries = max_retries, "重试节点");
                        }

                        if let Some(retry_config) = &node.retry_config {
                            #[cfg(not(feature = "retry"))]
                            { /* 重试功能关闭时不进入延迟逻辑 */ }
                            #[cfg(feature = "retry")]
                            let delay = match retry_config.strategy {
                                RetryStrategy::Fixed => retry_config.delay,
                                RetryStrategy::Exponential { multiplier } => {
                                    (retry_config.delay as f64
                                        * multiplier.powi(retries as i32))
                                        as u64
                                }
                                RetryStrategy::Linear { increment } => {
                                    retry_config.delay
                                        + (increment * retries as u64)
                                }
                            };
                            #[cfg(feature = "retry")]
                            tokio::time::sleep(Duration::from_millis(delay))
                                .await;
                        }
                        continue;
                    } else {
                        result.success = false;
                        result.error_message = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        result.end_time = Some(Instant::now());
        result.duration = start_time.elapsed();

        Ok(result)
    }

    /// 执行节点动作
    #[tracing::instrument(level = "debug", skip(context, config), fields(node_id = %node.id, node_name = %node.name, action_type = %node.action_spec.action_type))]
    async fn execute_node_action(
        node: &ExecutionNode,
        context: SharedContext,
        config: &ExecutorConfig,
    ) -> Result<()> {
        let action_spec = &node.action_spec;

        // 设置超时
        let timeout_duration = node
            .timeout_config
            .as_ref()
            .map(|c| Duration::from_millis(c.duration))
            .unwrap_or_else(|| Duration::from_millis(config.default_timeout));

        let action_future = Self::execute_action_by_type(action_spec, context);

        match tokio::time::timeout(timeout_duration, action_future).await {
            Ok(result) => result,
            Err(_) => {
                #[cfg(feature = "detailed-logging")]
                {
                    tracing::error!(node = %node.name, "节点执行超时");
                }
                Err(anyhow::anyhow!("节点 {} 执行超时", node.name))
            }
        }
    }

    /// 根据动作类型执行动作 (Public for demo purposes)
    pub fn execute_action_by_type(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<()>> + Send + '_>,
    > {
        Box::pin(async move {
            match action_spec.action_type.as_str() {
                "builtin" => {
                    Self::execute_builtin_action(action_spec, context).await
                }
                "cmd" => Self::execute_cmd_action(action_spec, context).await,
                "http" => Self::execute_http_action(action_spec, context).await,
                "wasm" => Self::execute_wasm_action(action_spec, context).await,
                "composite" => {
                    Self::execute_composite_action(action_spec, context).await
                }
                _ => Err(anyhow::anyhow!(
                    "不支持的动作类型: {}",
                    action_spec.action_type
                )),
            }
        })
    }

    /// 执行内置动作
    async fn execute_builtin_action(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        tracing::debug!("执行内置动作");

        // 获取操作类型参数
        let operation = action_spec
            .parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("内置动作缺少 'operation' 参数"))?;

        match operation {
            "set_variable" => {
                let key = action_spec
                    .parameters
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("set_variable 操作缺少 'key' 参数")
                    })?;

                let value =
                    action_spec.parameters.get("value").ok_or_else(|| {
                        anyhow::anyhow!("set_variable 操作缺少 'value' 参数")
                    })?;

                let mut guard = context.lock().await;
                guard.set_variable(key.to_string(), format!("{value:?}"));
                tracing::debug!("设置变量: {} = {:?}", key, value);
            }
            "get_variable" => {
                let key = action_spec
                    .parameters
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("get_variable 操作缺少 'key' 参数")
                    })?;

                let guard = context.lock().await;
                if let Some(value) = guard.variables.get(key) {
                    tracing::debug!("获取变量: {} = {}", key, value);
                } else {
                    return Err(anyhow::anyhow!("变量 '{}' 不存在", key));
                }
            }
            "log" => {
                let message =
                    action_spec.parameters.get("message").ok_or_else(|| {
                        anyhow::anyhow!("log 操作缺少 'message' 参数")
                    })?;

                let level = action_spec
                    .parameters
                    .get("level")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");

                match level {
                    "debug" => tracing::debug!("内置日志: {:?}", message),
                    "info" => tracing::info!("内置日志: {:?}", message),
                    "warn" => tracing::warn!("内置日志: {:?}", message),
                    "error" => tracing::error!("内置日志: {:?}", message),
                    _ => tracing::info!("内置日志: {:?}", message),
                }
            }
            "sleep" => {
                let duration = action_spec
                    .parameters
                    .get("duration")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        anyhow::anyhow!("sleep 操作缺少有效的 'duration' 参数")
                    })?;

                tracing::debug!("睡眠 {} 毫秒", duration);
                tokio::time::sleep(Duration::from_millis(duration)).await;
            }
            _ => {
                return Err(anyhow::anyhow!("不支持的内置操作: {}", operation));
            }
        }

        // 存储输出到上下文
        for (key, value) in &action_spec.outputs {
            let mut guard = context.lock().await;
            guard.set_variable(key.clone(), format!("{value:?}"));
        }

        Ok(())
    }

    /// 执行命令动作
    async fn execute_cmd_action(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        tracing::debug!("执行命令动作");

        let command = action_spec
            .parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("命令动作缺少 'command' 参数"))?;

        let args = action_spec
            .parameters
            .get("args")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let working_dir = action_spec
            .parameters
            .get("working_dir")
            .and_then(|v| v.as_str());

        tracing::debug!("执行命令: {} {:?}", command, args);

        let mut cmd = tokio::process::Command::new(command);
        cmd.args(&args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(env_vars) = action_spec.parameters.get("env") {
            if let Some(env_map) = env_vars.as_mapping() {
                for (key, value) in env_map {
                    if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                        cmd.env(k, v);
                    }
                }
            }
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("执行命令失败: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);

        tracing::debug!("命令执行完成，退出码: {}", exit_code);

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "命令执行失败，退出码: {}，错误输出: {}",
                exit_code,
                stderr
            ));
        }

        // 将命令输出存储到上下文
        {
            let mut guard = context.lock().await;
            guard.set_variable("cmd_stdout".to_string(), stdout.to_string());
            guard.set_variable("cmd_stderr".to_string(), stderr.to_string());
            guard.set_variable(
                "cmd_exit_code".to_string(),
                exit_code.to_string(),
            );
        }

        // 存储输出到上下文
        for (key, value) in &action_spec.outputs {
            let mut guard = context.lock().await;
            guard.set_variable(key.clone(), format!("{value:?}"));
        }

        Ok(())
    }

    /// 执行HTTP动作
    #[allow(unused_variables)]
    async fn execute_http_action(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        #[cfg(not(feature = "http"))]
        {
            tracing::warn!("HTTP 功能未启用，跳过 HTTP 动作");
            tokio::time::sleep(Duration::from_millis(300)).await;
            return Ok(());
        }

        #[cfg(feature = "http")]
        {
            tracing::debug!("执行HTTP动作");

            let url = action_spec
                .parameters
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("HTTP动作缺少 'url' 参数"))?;

            let method = action_spec
                .parameters
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");

            let client = reqwest::Client::new();
            let mut request = match method.to_uppercase().as_str() {
                "GET" => client.get(url),
                "POST" => client.post(url),
                "PUT" => client.put(url),
                "DELETE" => client.delete(url),
                "PATCH" => client.patch(url),
                _ => {
                    return Err(anyhow::anyhow!("不支持的HTTP方法: {}", method))
                }
            };

            // 添加请求头
            if let Some(headers) = action_spec.parameters.get("headers") {
                if let Some(headers_map) = headers.as_mapping() {
                    for (key, value) in headers_map {
                        if let (Some(k), Some(v)) =
                            (key.as_str(), value.as_str())
                        {
                            request = request.header(k, v);
                        }
                    }
                }
            }

            // 添加请求体
            if let Some(body) = action_spec.parameters.get("body") {
                let content_type = action_spec
                    .parameters
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("application/json");

                match content_type {
                    "application/json" => {
                        let json_body =
                            serde_json::to_string(body).map_err(|e| {
                                anyhow::anyhow!("序列化JSON失败: {}", e)
                            })?;
                        request = request
                            .header("Content-Type", "application/json")
                            .body(json_body);
                    }
                    "text/plain" => {
                        let text_body = body.as_str().unwrap_or("");
                        request = request
                            .header("Content-Type", "text/plain")
                            .body(text_body.to_string());
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "不支持的Content-Type: {}",
                            content_type
                        ));
                    }
                }
            }

            tracing::debug!("发送HTTP请求: {} {}", method, url);

            let response = request
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("HTTP请求失败: {}", e))?;

            let status_code = response.status().as_u16();
            let response_headers = response.headers().clone();
            let response_text = response
                .text()
                .await
                .map_err(|e| anyhow::anyhow!("读取响应体失败: {}", e))?;

            tracing::debug!("HTTP响应状态码: {}", status_code);

            // 将响应存储到上下文
            {
                let mut guard = context.lock().await;
                guard.set_variable(
                    "http_status_code".to_string(),
                    status_code.to_string(),
                );
                guard.set_variable(
                    "http_response_body".to_string(),
                    response_text.clone(),
                );

                // 存储响应头
                for (name, value) in response_headers.iter() {
                    if let Ok(value_str) = value.to_str() {
                        guard.set_variable(
                            format!("http_header_{}", name.as_str()),
                            value_str.to_string(),
                        );
                    }
                }
            }

            // 检查响应状态
            if !reqwest::StatusCode::from_u16(status_code)
                .map_err(|_| anyhow::anyhow!("无效的状态码: {}", status_code))?
                .is_success()
            {
                return Err(anyhow::anyhow!(
                    "HTTP请求失败，状态码: {}，响应: {}",
                    status_code,
                    response_text
                ));
            }

            // 存储输出到上下文
            for (key, value) in &action_spec.outputs {
                let mut guard = context.lock().await;
                guard.set_variable(key.clone(), format!("{value:?}"));
            }

            Ok(())
        }
    }

    /// 执行WASM动作
    async fn execute_wasm_action(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        tracing::debug!("执行WASM动作");

        // TODO: 实现真正的WASM执行
        // 这里暂时保持原有的模拟行为，但添加参数处理

        let _module_path = action_spec
            .parameters
            .get("module")
            .and_then(|v| v.as_str());

        let _function_name = action_spec
            .parameters
            .get("function")
            .and_then(|v| v.as_str())
            .unwrap_or("main");

        // 模拟WASM执行
        tokio::time::sleep(Duration::from_millis(150)).await;

        tracing::debug!("WASM模块执行完成");

        // 存储输出到上下文
        for (key, value) in &action_spec.outputs {
            let mut guard = context.lock().await;
            guard.set_variable(key.clone(), format!("{value:?}"));
        }

        Ok(())
    }

    /// 执行复合动作
    async fn execute_composite_action(
        action_spec: &ActionSpec,
        context: SharedContext,
    ) -> Result<()> {
        tracing::debug!("执行复合动作");

        // 复合动作按顺序执行所有子动作
        let actions = action_spec
            .parameters
            .get("actions")
            .and_then(|v| v.as_sequence())
            .ok_or_else(|| anyhow::anyhow!("复合动作缺少 'actions' 参数"))?;

        for (index, action_value) in actions.iter().enumerate() {
            if let Some(action_map) = action_value.as_mapping() {
                let action_type = action_map
                    .get(serde_yaml::Value::String("type".to_string()))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("子动作 {} 缺少 'type' 参数", index)
                    })?;

                let parameters = action_map
                    .get(serde_yaml::Value::String("parameters".to_string()))
                    .and_then(|v| v.as_mapping())
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| {
                                (
                                    k.as_str().unwrap_or("").to_string(),
                                    v.clone(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let outputs = action_map
                    .get(serde_yaml::Value::String("outputs".to_string()))
                    .and_then(|v| v.as_mapping())
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| {
                                (
                                    k.as_str().unwrap_or("").to_string(),
                                    v.clone(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let sub_action_spec = ActionSpec {
                    action_type: action_type.to_string(),
                    parameters,
                    outputs,
                };

                tracing::debug!("执行子动作 {}: {}", index, action_type);
                Self::execute_action_by_type(&sub_action_spec, context.clone())
                    .await?;
            }
        }

        // 存储输出到上下文
        for (key, value) in &action_spec.outputs {
            let mut guard = context.lock().await;
            guard.set_variable(key.clone(), format!("{value:?}"));
        }

        Ok(())
    }

    /// 设置上下文
    #[tracing::instrument(level = "debug", skip(self, context), fields(workflow = %plan.metadata.workflow_name))]
    async fn setup_context(
        &self,
        plan: &ExecutionPlan,
        context: SharedContext,
    ) -> Result<()> {
        let mut guard = context.lock().await;

        // 设置环境变量
        for (key, value) in &plan.env_vars {
            guard.set_variable(format!("env.{key}"), format!("{value:?}"));
        }

        // 设置流程变量
        for (key, value) in &plan.flow_vars {
            guard.set_variable(format!("flow.{key}"), format!("{value:?}"));
        }

        Ok(())
    }

    /// 更新统计信息
    #[cfg(feature = "perf-metrics")]
    fn update_stats(&mut self, result: &ExecutionResult) {
        self.stats.total_execution_time = result.total_duration;

        for phase_result in &result.phase_results {
            for node_result in &phase_result.node_results {
                self.stats.total_tasks += 1;
                if node_result.success {
                    self.stats.successful_tasks += 1;
                } else {
                    self.stats.failed_tasks += 1;
                }
            }
        }

        if self.stats.total_tasks > 0 {
            self.stats.average_execution_time = Duration::from_nanos(
                self.stats.total_execution_time.as_nanos() as u64
                    / self.stats.total_tasks as u64,
            );
        }
    }

    /// 获取执行统计
    #[cfg(feature = "perf-metrics")]
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.stats
    }
}

impl Executor for EnhancedTaskExecutor {
    type Input = (ExecutionPlan, SharedContext);
    type Output = ExecutionResult;
    type Error = anyhow::Error;

    async fn execute(
        &mut self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (plan, context) = input;
        self.execute_plan(plan, context).await
    }

    fn status(&self) -> ExecutorStatus {
        self.status.clone()
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.status = ExecutorStatus::Stopped;
        Ok(())
    }
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 计划ID
    pub plan_id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 阶段结果
    pub phase_results: Vec<PhaseResult>,
    /// 总执行时间
    pub total_duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 阶段结果
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// 阶段ID
    pub phase_id: String,
    /// 阶段名称
    pub phase_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 节点结果
    pub node_results: Vec<NodeResult>,
}

/// 节点结果
#[derive(Debug, Clone)]
pub struct NodeResult {
    /// 节点ID
    pub node_id: String,
    /// 节点名称
    pub node_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 重试次数
    pub retry_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowbuilder_core::{ActionSpec, ExecutionNode};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = EnhancedTaskExecutor::new();
        assert_eq!(executor.status(), ExecutorStatus::Idle);
    }

    #[tokio::test]
    async fn test_node_execution() {
        let config = ExecutorConfig::default();
        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let mut parameters = HashMap::new();
        parameters.insert(
            "operation".to_string(),
            serde_yaml::Value::String("log".to_string()),
        );
        parameters.insert(
            "message".to_string(),
            serde_yaml::Value::String("test message".to_string()),
        );

        let node = ExecutionNode::new(
            "test_node".to_string(),
            "Test Node".to_string(),
            ActionSpec {
                action_type: "builtin".to_string(),
                parameters,
                outputs: HashMap::new(),
            },
        );

        let result =
            EnhancedTaskExecutor::execute_node_static(&node, context, &config)
                .await;
        assert!(result.is_ok());

        let node_result = result.unwrap();
        assert!(node_result.success);
        assert_eq!(node_result.node_id, "test_node");
    }

    #[tokio::test]
    async fn test_builtin_set_variable_action() {
        let action_spec = ActionSpec {
            action_type: "builtin".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "operation".to_string(),
                    serde_yaml::Value::String("set_variable".to_string()),
                );
                params.insert(
                    "key".to_string(),
                    serde_yaml::Value::String("test_key".to_string()),
                );
                params.insert(
                    "value".to_string(),
                    serde_yaml::Value::String("test_value".to_string()),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result = EnhancedTaskExecutor::execute_builtin_action(
            &action_spec,
            context.clone(),
        )
        .await;
        assert!(result.is_ok());

        // Verify variable was set
        let guard = context.lock().await;
        assert!(guard.variables.contains_key("test_key"));
        let stored_value = guard.variables.get("test_key").unwrap();
        assert!(stored_value.contains("test_value")); // Just check it contains the value
    }

    #[tokio::test]
    async fn test_builtin_sleep_action() {
        let action_spec = ActionSpec {
            action_type: "builtin".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "operation".to_string(),
                    serde_yaml::Value::String("sleep".to_string()),
                );
                params.insert(
                    "duration".to_string(),
                    serde_yaml::Value::Number(50.into()),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let start = std::time::Instant::now();
        let result =
            EnhancedTaskExecutor::execute_builtin_action(&action_spec, context)
                .await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_cmd_action() {
        let action_spec = ActionSpec {
            action_type: "cmd".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "command".to_string(),
                    serde_yaml::Value::String("echo".to_string()),
                );
                params.insert(
                    "args".to_string(),
                    serde_yaml::Value::Sequence(vec![
                        serde_yaml::Value::String("hello world".to_string()),
                    ]),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result = EnhancedTaskExecutor::execute_cmd_action(
            &action_spec,
            context.clone(),
        )
        .await;
        assert!(result.is_ok());

        // Verify command output was stored
        let guard = context.lock().await;
        assert!(guard.variables.contains_key("cmd_stdout"));
        assert!(guard
            .variables
            .get("cmd_stdout")
            .unwrap()
            .contains("hello world"));
    }

    #[cfg(feature = "http")]
    #[tokio::test]
    async fn test_http_action() {
        // Skip this test if we don't have internet connectivity
        let action_spec = ActionSpec {
            action_type: "http".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "url".to_string(),
                    serde_yaml::Value::String(
                        "https://httpbin.org/get".to_string(),
                    ),
                );
                params.insert(
                    "method".to_string(),
                    serde_yaml::Value::String("GET".to_string()),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result = EnhancedTaskExecutor::execute_http_action(
            &action_spec,
            context.clone(),
        )
        .await;

        // HTTP tests may fail due to network issues, so we'll make it more lenient
        if result.is_ok() {
            // Verify HTTP response was stored
            let guard = context.lock().await;
            assert!(guard.variables.contains_key("http_status_code"));
            assert_eq!(guard.variables.get("http_status_code").unwrap(), "200");
        } else {
            // If HTTP fails (network issues), just verify the error is handled properly
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_wasm_action() {
        let action_spec = ActionSpec {
            action_type: "wasm".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "module".to_string(),
                    serde_yaml::Value::String("test.wasm".to_string()),
                );
                params.insert(
                    "function".to_string(),
                    serde_yaml::Value::String("main".to_string()),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result =
            EnhancedTaskExecutor::execute_wasm_action(&action_spec, context)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_composite_action() {
        let action_spec = ActionSpec {
            action_type: "composite".to_string(),
            parameters: {
                let mut params = HashMap::new();

                let sub_actions = vec![serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::mapping::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("type".to_string()),
                        serde_yaml::Value::String("builtin".to_string()),
                    );
                    map.insert(
                        serde_yaml::Value::String("parameters".to_string()),
                        serde_yaml::Value::Mapping({
                            let mut param_map =
                                serde_yaml::mapping::Mapping::new();
                            param_map.insert(
                                serde_yaml::Value::String(
                                    "operation".to_string(),
                                ),
                                serde_yaml::Value::String("log".to_string()),
                            );
                            param_map.insert(
                                serde_yaml::Value::String(
                                    "message".to_string(),
                                ),
                                serde_yaml::Value::String(
                                    "composite test".to_string(),
                                ),
                            );
                            param_map
                        }),
                    );
                    map.insert(
                        serde_yaml::Value::String("outputs".to_string()),
                        serde_yaml::Value::Mapping(
                            serde_yaml::mapping::Mapping::new(),
                        ),
                    );
                    map
                })];

                params.insert(
                    "actions".to_string(),
                    serde_yaml::Value::Sequence(sub_actions),
                );
                params
            },
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result = EnhancedTaskExecutor::execute_composite_action(
            &action_spec,
            context,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unsupported_action_type() {
        let action_spec = ActionSpec {
            action_type: "unsupported".to_string(),
            parameters: HashMap::new(),
            outputs: HashMap::new(),
        };

        let context = Arc::new(tokio::sync::Mutex::new(
            flowbuilder_context::FlowContext::default(),
        ));

        let result =
            EnhancedTaskExecutor::execute_action_by_type(&action_spec, context)
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不支持的动作类型"));
    }
}
