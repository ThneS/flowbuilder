use crate::context::{FlowContext, SharedContext};
use anyhow::{Result, anyhow};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::sync::Mutex;

type StepFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub type Step = Box<dyn FnOnce(SharedContext) -> StepFuture + Send>;

pub struct FlowBuilder {
    pub steps: Vec<Step>,
}

impl Default for FlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn step<Fut, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| Box::pin(f(ctx))));
        self
    }

    pub fn named_step<Fut, F>(mut self, name: &'static str, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = f(ctx2.clone()).await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    pub fn step_if<Fut, F, Cond>(mut self, cond: Cond, mut f: F) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let guard = ctx2.lock().await;
                if cond(&guard) {
                    drop(guard);
                    f(ctx2).await
                } else {
                    let trace_id = guard.trace_id.clone();
                    drop(guard);
                    println!("[trace_id:{}] [step_if] condition not met, skipping step", trace_id);
                    Ok(())
                }
            })
        }));
        self
    }

    pub fn wait_until<Cond>(mut self, cond: Cond, interval: Duration, max_retry: usize) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                for attempt in 0..max_retry {
                    {
                        let guard = ctx.lock().await;
                        if cond(&guard) {
                            println!("[wait_until] condition met on attempt {}", attempt + 1);
                            return Ok(());
                        }
                    }
                    println!(
                        "[wait_until] attempt {}/{} failed, waiting...",
                        attempt + 1,
                        max_retry
                    );
                    tokio::time::sleep(interval).await;
                }
                Err(anyhow!(
                    "wait_until: condition not met after {} retries",
                    max_retry
                ))
            })
        }));
        self
    }

    pub fn step_handle_error<Fut, F, H>(
        mut self,
        name: &'static str,
        mut f: F,
        mut handle: H,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        H: FnMut(&mut FlowContext, anyhow::Error) -> Result<()> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                println!("[step:{}] running...", name);
                match f(ctx2.clone()).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        println!("[step:{}] error: {}", name, e);
                        let mut guard = ctx2.lock().await;
                        handle(&mut guard, e)
                    }
                }
            })
        }));
        self
    }

    pub fn subflow_if<Cond, G>(mut self, cond: Cond, subflow_gen: G) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        G: Fn() -> FlowBuilder + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let guard = ctx2.lock().await;
                if cond(&guard) {
                    drop(guard);
                    let subflow = subflow_gen();
                    subflow.run_all_with_context(ctx2.clone()).await
                } else {
                    Ok(())
                }
            })
        }));
        self
    }

    // 添加超时处理的 step
    pub fn step_with_timeout<Fut, F>(
        mut self,
        name: &'static str,
        timeout: Duration,
        mut f: F,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = match tokio::time::timeout(timeout, f(ctx2.clone())).await {
                    Ok(result) => result,
                    Err(_) => {
                        // 超时处理
                        {
                            let mut guard = ctx2.lock().await;
                            guard.end_step_timeout(name);
                        }
                        Err(anyhow!("step {} timed out after {:?}", name, timeout))
                    }
                };

                // 结束记录步骤（非超时情况）
                if result.is_ok() || !matches!(result, Err(ref e) if e.to_string().contains("timed out")) {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    // 添加全流程级别的超时控制
    pub async fn run_all_with_timeout(self, timeout: Duration) -> Result<()> {
        let ctx = Arc::new(Mutex::new(FlowContext::default()));
        match tokio::time::timeout(timeout, self.run_all_with_context(ctx)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Flow execution timed out after {:?}", timeout))
        }
    }

    pub async fn run_all_with_timeout_and_trace_id(self, timeout: Duration, trace_id: String) -> Result<()> {
        let ctx = Arc::new(Mutex::new(FlowContext::new_with_trace_id(trace_id)));
        match tokio::time::timeout(timeout, self.run_all_with_context(ctx)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Flow execution timed out after {:?}", timeout))
        }
    }

    // 添加重试机制的 step
    pub fn step_with_retry<Fut, F>(
        mut self,
        name: &'static str,
        max_retries: usize,
        retry_delay: Duration,
        mut f: F,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                println!(
                    "[step:{}] running with max {} retries...",
                    name, max_retries
                );

                for attempt in 0..=max_retries {
                    match f(ctx.clone()).await {
                        Ok(()) => {
                            if attempt > 0 {
                                println!("[step:{}] succeeded on attempt {}", name, attempt + 1);
                            }
                            return Ok(());
                        }
                        Err(e) => {
                            if attempt < max_retries {
                                println!(
                                    "[step:{}] attempt {}/{} failed: {}, retrying in {:?}...",
                                    name,
                                    attempt + 1,
                                    max_retries + 1,
                                    e,
                                    retry_delay
                                );
                                tokio::time::sleep(retry_delay).await;
                            } else {
                                println!("[step:{}] all {} attempts failed", name, max_retries + 1);
                                return Err(e);
                            }
                        }
                    }
                }
                Ok(())
            })
        }));
        self
    }

    // 添加并行执行的 step
    pub fn parallel_steps<F>(mut self, subflows: Vec<F>) -> Self
    where
        F: Fn() -> FlowBuilder + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                let trace_id = {
                    let guard = ctx2.lock().await;
                    guard.trace_id.clone()
                };

                println!("[trace_id:{}] [parallel_steps] executing {} parallel flows...",
                        trace_id, subflows.len());

                let tasks: Vec<_> = subflows
                    .into_iter()
                    .enumerate()
                    .map(|(i, subflow_gen)| {
                        let ctx_clone = ctx2.clone();
                        tokio::spawn(async move {
                            println!("[parallel_steps] starting parallel flow {}", i);
                            let result = subflow_gen().run_all_with_context(ctx_clone).await;
                            if let Err(ref e) = result {
                                println!("[parallel_steps] parallel flow {} failed: {}", i, e);
                            }
                            (i, result)
                        })
                    })
                    .collect();

                for task in tasks {
                    let (task_id, result) = task.await
                        .map_err(|e| anyhow!("parallel task panicked: {}", e))?;
                    result.map_err(|e| anyhow!("parallel task {} failed: {}", task_id, e))?;
                }

                println!("[trace_id:{}] [parallel_steps] all parallel flows completed", trace_id);
                Ok(())
            })
        }));
        self
    }

    // 添加带结果收集的并行执行
    pub fn parallel_steps_with_join<F>(mut self, name: &'static str, subflows: Vec<F>) -> Self
    where
        F: Fn() -> FlowBuilder + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = async {
                    let trace_id = {
                        let guard = ctx2.lock().await;
                        guard.trace_id.clone()
                    };

                    println!("[trace_id:{}] [{}] executing {} parallel flows with join...",
                            trace_id, name, subflows.len());

                    let tasks: Vec<_> = subflows
                        .into_iter()
                        .enumerate()
                        .map(|(i, subflow_gen)| {
                            let ctx_clone = ctx2.clone();
                            tokio::spawn(async move {
                                println!("[{}] starting parallel flow {}", name, i);
                                let result = subflow_gen().run_all_with_context(ctx_clone).await;
                                if let Err(ref e) = result {
                                    println!("[{}] parallel flow {} failed: {}", name, i, e);
                                }
                                (i, result)
                            })
                        })
                        .collect();

                    let mut success_count = 0;
                    let mut failed_count = 0;

                    for task in tasks {
                        let (i, result) = task.await
                            .map_err(|e| anyhow!("parallel task panicked: {}", e))?;

                        match result {
                            Ok(()) => {
                                success_count += 1;
                                println!("[{}] parallel flow {} completed successfully", name, i);
                            }
                            Err(e) => {
                                failed_count += 1;
                                println!("[{}] parallel flow {} failed: {}", name, i, e);
                            }
                        }
                    }

                    // 将结果存储到上下文中
                    {
                        let mut guard = ctx2.lock().await;
                        guard.set_variable(format!("{}_parallel_success", name), success_count.to_string());
                        guard.set_variable(format!("{}_parallel_failed", name), failed_count.to_string());
                        guard.set_variable(format!("{}_parallel_total", name), (success_count + failed_count).to_string());
                    }

                    println!("[trace_id:{}] [{}] parallel execution completed: {} success, {} failed",
                            trace_id, name, success_count, failed_count);

                    if failed_count > 0 {
                        Err(anyhow!("parallel execution had {} failures out of {} tasks",
                                   failed_count, success_count + failed_count))
                    } else {
                        Ok(())
                    }
                }.await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    // 添加循环执行的 step
    pub fn step_while<Fut, F, Cond>(
        mut self,
        name: &'static str,
        cond: Cond,
        max_iterations: usize,
        mut f: F,
    ) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                println!(
                    "[step_while:{}] starting loop with max {} iterations...",
                    name, max_iterations
                );

                let mut iteration = 0;
                while iteration < max_iterations {
                    {
                        let guard = ctx.lock().await;
                        if !cond(&guard) {
                            println!(
                                "[step_while:{}] condition not met, exiting loop at iteration {}",
                                name, iteration
                            );
                            break;
                        }
                    }

                    println!("[step_while:{}] iteration {}", name, iteration + 1);
                    f(ctx.clone()).await?;
                    iteration += 1;
                }

                if iteration >= max_iterations {
                    println!(
                        "[step_while:{}] reached max iterations {}",
                        name, max_iterations
                    );
                }

                Ok(())
            })
        }));
        self
    }

    // 添加条件分支的 step
    pub fn step_switch<Cond, G>(
        mut self,
        condition_branches: Vec<(Cond, G)>,
        default_branch: Option<G>,
    ) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        G: Fn() -> FlowBuilder + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                println!(
                    "[step_switch] evaluating {} conditions...",
                    condition_branches.len()
                );

                let guard = ctx.lock().await;

                // 检查条件分支
                for (i, (cond, subflow_gen)) in condition_branches.into_iter().enumerate() {
                    if cond(&guard) {
                        drop(guard);
                        println!("[step_switch] condition {} matched, executing branch", i);
                        return subflow_gen().run_all_with_context(ctx.clone()).await;
                    }
                }

                // 执行默认分支
                if let Some(default_gen) = default_branch {
                    drop(guard);
                    println!("[step_switch] no conditions matched, executing default branch");
                    default_gen().run_all_with_context(ctx.clone()).await
                } else {
                    drop(guard);
                    println!("[step_switch] no conditions matched and no default branch");
                    Ok(())
                }
            })
        }));
        self
    }

    // 添加错误不中断流程的 step
    pub fn step_continue_on_error<Fut, F>(mut self, name: &'static str, mut f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = f(ctx2.clone()).await;

                // 记录步骤结果，但错误不中断流程
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => {
                            guard.end_step_failed(name, &e.to_string());
                            // 设置 ok 为 false 但不返回错误，继续执行
                            guard.ok = false;
                        }
                    }
                }

                // 总是返回 Ok，错误已记录在 context 中
                Ok(())
            })
        }));
        self
    }

    // 添加循环等待直到条件满足的 step
    pub fn step_wait_until<Cond>(
        mut self,
        name: &'static str,
        cond: Cond,
        interval: Duration,
        max_retry: usize
    ) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = async {
                    for attempt in 0..max_retry {
                        {
                            let guard = ctx2.lock().await;
                            if cond(&guard) {
                                println!("[trace_id:{}] [step:{}] condition met on attempt {}",
                                        guard.trace_id, name, attempt + 1);
                                return Ok(());
                            }
                        }
                        println!("[step:{}] attempt {}/{} failed, waiting {:?}...",
                                name, attempt + 1, max_retry, interval);
                        tokio::time::sleep(interval).await;
                    }
                    Err(anyhow!("step_wait_until: condition not met after {} retries", max_retry))
                }.await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    // 添加创建快照的 step
    pub fn create_snapshot(mut self, snapshot_id: &'static str, description: &'static str) -> Self {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                let mut guard = ctx.lock().await;
                guard.start_step(format!("create_snapshot_{}", snapshot_id));

                let result = match guard.create_snapshot(snapshot_id.to_string(), description.to_string()) {
                    Ok(()) => {
                        guard.end_step_success(&format!("create_snapshot_{}", snapshot_id));
                        Ok(())
                    }
                    Err(e) => {
                        guard.end_step_failed(&format!("create_snapshot_{}", snapshot_id), &e);
                        Err(anyhow!(e))
                    }
                };

                result
            })
        }));
        self
    }

    // 添加回滚到快照的 step
    pub fn rollback_to_snapshot(mut self, snapshot_id: &'static str) -> Self {
        self.steps.push(Box::new(move |ctx| {
            Box::pin(async move {
                let mut guard = ctx.lock().await;
                guard.start_step(format!("rollback_to_{}", snapshot_id));

                let result = match guard.rollback_to_snapshot(snapshot_id) {
                    Ok(()) => {
                        guard.end_step_success(&format!("rollback_to_{}", snapshot_id));
                        Ok(())
                    }
                    Err(e) => {
                        guard.end_step_failed(&format!("rollback_to_{}", snapshot_id), &e);
                        Err(anyhow!(e))
                    }
                };

                result
            })
        }));
        self
    }

    // 添加带自动回滚的 step
    pub fn step_with_rollback<Fut, F>(
        mut self,
        name: &'static str,
        snapshot_id: &'static str,
        mut f: F,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 创建快照
                {
                    let mut guard = ctx2.lock().await;
                    guard.create_snapshot(
                        snapshot_id.to_string(),
                        format!("Auto snapshot before step: {}", name)
                    ).map_err(|e| anyhow!("Failed to create snapshot: {}", e))?;
                    guard.start_step(name.to_string());
                }

                // 执行步骤
                let result = f(ctx2.clone()).await;

                // 处理结果
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => {
                            guard.end_step_success(name);
                            // 成功时删除快照
                            let _ = guard.remove_snapshot(snapshot_id);
                        }
                        Err(e) => {
                            guard.end_step_failed(name, &e.to_string());
                            // 失败时回滚
                            println!("[trace_id:{}] Step '{}' failed, rolling back...", guard.trace_id, name);
                            if let Err(rollback_err) = guard.rollback_to_snapshot(snapshot_id) {
                                println!("[trace_id:{}] Rollback failed: {}", guard.trace_id, rollback_err);
                                // 回滚失败时，返回复合错误
                                return Err(anyhow!("Step '{}' failed: {}, and rollback also failed: {}", 
                                    name, e, rollback_err));
                            } else {
                                println!("[trace_id:{}] Successfully rolled back to snapshot '{}'", guard.trace_id, snapshot_id);
                                // 回滚成功时，允许流程继续，不传播原始错误
                                return Ok(());
                            }
                        }
                    }
                }

                result
            })
        }));
        self
    }

    // 添加条件回滚的 step
    pub fn step_with_conditional_rollback<Fut, F, Cond>(
        mut self,
        name: &'static str,
        snapshot_id: &'static str,
        mut f: F,
        rollback_condition: Cond,
    ) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 创建快照
                {
                    let mut guard = ctx2.lock().await;
                    guard.create_snapshot(
                        snapshot_id.to_string(),
                        format!("Conditional snapshot before step: {}", name)
                    ).map_err(|e| anyhow!("Failed to create snapshot: {}", e))?;
                    guard.start_step(name.to_string());
                }

                // 执行步骤
                let result = f(ctx2.clone()).await;

                // 检查是否需要回滚
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => {
                            if rollback_condition(&guard) {
                                println!("[trace_id:{}] Rollback condition met for step '{}', rolling back...",
                                        guard.trace_id, name);
                                if let Err(rollback_err) = guard.rollback_to_snapshot(snapshot_id) {
                                    println!("[trace_id:{}] Rollback failed: {}", guard.trace_id, rollback_err);
                                }
                                guard.end_step_success(name);
                            } else {
                                guard.end_step_success(name);
                                let _ = guard.remove_snapshot(snapshot_id);
                            }
                        }
                        Err(e) => {
                            guard.end_step_failed(name, &e.to_string());
                            // 失败时总是回滚
                            if let Err(rollback_err) = guard.rollback_to_snapshot(snapshot_id) {
                                println!("[trace_id:{}] Rollback failed: {}", guard.trace_id, rollback_err);
                            }
                        }
                    }
                }

                result
            })
        }));
        self
    }

    pub async fn run_all(self) -> Result<()> {
        self.run_all_with_context(Arc::new(Mutex::new(FlowContext::default())))
            .await
    }

    pub async fn run_all_with_context(self, ctx: SharedContext) -> Result<()> {
        let trace_id = {
            let guard = ctx.lock().await;
            guard.trace_id.clone()
        };

        println!("[trace_id:{}] Starting flow execution with {} steps", trace_id, self.steps.len());

        for (i, step) in self.steps.into_iter().enumerate() {
            println!("[trace_id:{}] Executing step {}/{}", trace_id, i + 1, i + 1);
            step(ctx.clone()).await?;
        }

        // 打印流程摘要
        {
            let guard = ctx.lock().await;
            guard.print_summary();
        }

        println!("[trace_id:{}] Flow execution completed", trace_id);
        Ok(())
    }

    pub async fn run_all_with_trace_id(self, trace_id: String) -> Result<()> {
        let ctx = Arc::new(Mutex::new(FlowContext::new_with_trace_id(trace_id)));
        self.run_all_with_context(ctx).await
    }

    // 添加多路分支 switch-case 功能
    pub fn step_switch_match<F, G>(mut self, name: &'static str, matcher: F) -> Self
    where
        F: Fn(&FlowContext) -> Option<G> + Send + Sync + 'static,
        G: Fn() -> FlowBuilder + Send + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = async {
                    let guard = ctx2.lock().await;
                    let trace_id = guard.trace_id.clone();
                    
                    if let Some(flow_generator) = matcher(&guard) {
                        println!("[trace_id:{}] [{}] condition matched, executing branch", trace_id, name);
                        drop(guard);
                        flow_generator().run_all_with_context(ctx2.clone()).await
                    } else {
                        println!("[trace_id:{}] [{}] no condition matched, skipping", trace_id, name);
                        drop(guard);
                        Ok(())
                    }
                }.await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    // 添加支持 boxed closures 的多路分支 switch-case 功能
    pub fn step_switch_match_boxed<F>(mut self, name: &'static str, matcher: F) -> Self
    where
        F: Fn(&FlowContext) -> Option<Box<dyn Fn() -> FlowBuilder + Send>> + Send + Sync + 'static,
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = async {
                    let guard = ctx2.lock().await;
                    let trace_id = guard.trace_id.clone();
                    
                    if let Some(flow_generator) = matcher(&guard) {
                        println!("[trace_id:{}] [{}] condition matched, executing branch", trace_id, name);
                        drop(guard);
                        flow_generator().run_all_with_context(ctx2.clone()).await
                    } else {
                        println!("[trace_id:{}] [{}] no condition matched, skipping", trace_id, name);
                        drop(guard);
                        Ok(())
                    }
                }.await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }

    // 添加全局错误处理器
    pub fn with_global_error_handler<H>(self, handler: H) -> FlowBuilderWithErrorHandler<H>
    where
        H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
    {
        FlowBuilderWithErrorHandler {
            inner: self,
            error_handler: handler,
        }
    }

    // 添加支持3参数的全局错误处理器 (step_name, context, error) -> bool (是否继续)
    pub fn with_global_error_handler_advanced<H>(self, handler: H) -> FlowBuilderWithErrorHandlerAdvanced<H>
    where
        H: Fn(&str, &mut FlowContext, anyhow::Error) -> bool + Send + Sync + 'static,
    {
        FlowBuilderWithErrorHandlerAdvanced {
            inner: self,
            error_handler: handler,
        }
    }

    // 添加可恢复的执行方法（包装所有步骤以使用全局错误处理）
    pub async fn run_all_with_recovery<H>(self, error_handler: H) -> Result<()>
    where
        H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
    {
        let ctx = Arc::new(Mutex::new(FlowContext::default()));
        self.run_all_with_context_and_recovery(ctx, error_handler).await
    }

    pub async fn run_all_with_context_and_recovery<H>(self, ctx: SharedContext, error_handler: H) -> Result<()>
    where
        H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
    {
        let trace_id = {
            let guard = ctx.lock().await;
            guard.trace_id.clone()
        };

        println!("[trace_id:{}] Starting flow execution with global error handler, {} steps", trace_id, self.steps.len());

        for (i, step) in self.steps.into_iter().enumerate() {
            println!("[trace_id:{}] Executing step {}/{}", trace_id, i + 1, i + 1);
            
            match step(ctx.clone()).await {
                Ok(()) => {
                    // 步骤成功，继续执行
                }
                Err(e) => {
                    // 使用全局错误处理器处理错误
                    println!("[trace_id:{}] Step {}/{} failed, invoking global error handler", trace_id, i + 1, i + 1);
                    let mut guard = ctx.lock().await;
                    match error_handler(&mut guard, e) {
                        Ok(()) => {
                            println!("[trace_id:{}] Global error handler resolved the error, continuing flow", trace_id);
                            // 错误已处理，继续执行下一步
                        }
                        Err(handler_error) => {
                            println!("[trace_id:{}] Global error handler failed: {}", trace_id, handler_error);
                            drop(guard);
                            // 打印流程摘要
                            {
                                let guard = ctx.lock().await;
                                guard.print_summary();
                            }
                            return Err(handler_error);
                        }
                    }
                }
            }
        }

        // 打印流程摘要
        {
            let guard = ctx.lock().await;
            guard.print_summary();
        }

        println!("[trace_id:{}] Flow execution completed with global error handling", trace_id);
        Ok(())
    }

    // 添加带全局错误处理和自定义 trace ID 的执行方法
    pub async fn run_all_with_recovery_and_trace_id<H>(self, error_handler: H, trace_id: String) -> Result<()>
    where
        H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
    {
        let ctx = Arc::new(Mutex::new(FlowContext::new_with_trace_id(trace_id)));
        self.run_all_with_context_and_recovery(ctx, error_handler).await
    }

    // 添加简化的字符串匹配 switch-case (支持 boxed closures)
    pub fn step_switch_str(mut self, name: &'static str, variable_key: &'static str, cases: Vec<(&'static str, Box<dyn Fn() -> FlowBuilder + Send>)>, default_case: Option<Box<dyn Fn() -> FlowBuilder + Send>>) -> Self
    {
        self.steps.push(Box::new(move |ctx| {
            let ctx2 = ctx.clone();
            Box::pin(async move {
                // 开始记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    guard.start_step(name.to_string());
                }

                let result = async {
                    let guard = ctx2.lock().await;
                    let trace_id = guard.trace_id.clone();
                    let value = guard.get_variable(variable_key).map(|s| s.as_str()).unwrap_or("");
                    
                    // 查找匹配的分支
                    for (case_value, flow_generator) in cases.into_iter() {
                        if value == case_value {
                            println!("[trace_id:{}] [{}] matched case '{}', executing branch", trace_id, name, case_value);
                            drop(guard);
                            return flow_generator().run_all_with_context(ctx2.clone()).await;
                        }
                    }
                    
                    // 执行默认分支
                    if let Some(default_generator) = default_case {
                        println!("[trace_id:{}] [{}] no case matched, executing default branch", trace_id, name);
                        drop(guard);
                        default_generator().run_all_with_context(ctx2.clone()).await
                    } else {
                        println!("[trace_id:{}] [{}] no case matched and no default branch", trace_id, name);
                        drop(guard);
                        Ok(())
                    }
                }.await;

                // 结束记录步骤
                {
                    let mut guard = ctx2.lock().await;
                    match &result {
                        Ok(()) => guard.end_step_success(name),
                        Err(e) => guard.end_step_failed(name, &e.to_string()),
                    }
                }

                result
            })
        }));
        self
    }
}

// 带全局错误处理器的 FlowBuilder 包装器
pub struct FlowBuilderWithErrorHandler<H>
where
    H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
{
    inner: FlowBuilder,
    error_handler: H,
}

impl<H> FlowBuilderWithErrorHandler<H>
where
    H: Fn(&mut FlowContext, anyhow::Error) -> Result<()> + Send + Sync + 'static,
{
    // 继续添加步骤
    pub fn named_step<Fut, F>(mut self, name: &'static str, f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.inner = self.inner.named_step(name, f);
        self
    }

    pub fn step_if<Fut, F, Cond>(mut self, cond: Cond, f: F) -> Self
    where
        Cond: Fn(&FlowContext) -> bool + Send + Sync + 'static,
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.inner = self.inner.step_if(cond, f);
        self
    }

    // 执行方法
    pub async fn run_all(self) -> Result<()> {
        self.inner.run_all_with_recovery(self.error_handler).await
    }

    pub async fn run_all_with_trace_id(self, trace_id: String) -> Result<()> {
        self.inner.run_all_with_recovery_and_trace_id(self.error_handler, trace_id).await
    }

    pub async fn run_all_with_context(self, ctx: SharedContext) -> Result<()> {
        self.inner.run_all_with_context_and_recovery(ctx, self.error_handler).await
    }
}

// 带高级全局错误处理器的 FlowBuilder 包装器
pub struct FlowBuilderWithErrorHandlerAdvanced<H>
where
    H: Fn(&str, &mut FlowContext, anyhow::Error) -> bool + Send + Sync + 'static,
{
    inner: FlowBuilder,
    error_handler: H,
}

impl<H> FlowBuilderWithErrorHandlerAdvanced<H>
where
    H: Fn(&str, &mut FlowContext, anyhow::Error) -> bool + Send + Sync + 'static,
{
    // 继续添加步骤
    pub fn named_step<Fut, F>(mut self, name: &'static str, f: F) -> Self
    where
        F: FnMut(SharedContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.inner = self.inner.named_step(name, f);
        self
    }

    // 执行方法
    pub async fn run_all_with_recovery(self, trace_id: String) -> Result<()> {
        let ctx = Arc::new(Mutex::new(FlowContext::new_with_trace_id(trace_id.clone())));
        
        for (i, step) in self.inner.steps.into_iter().enumerate() {
            let step_name = format!("step_{}", i + 1);
            match step(ctx.clone()).await {
                Ok(()) => {
                    // 步骤成功
                }
                Err(e) => {
                    // 调用高级错误处理器
                    let should_continue = {
                        let mut guard = ctx.lock().await;
                        (self.error_handler)(&step_name, &mut guard, e)
                    };
                    
                    if !should_continue {
                        return Err(anyhow::anyhow!("Flow stopped by error handler at step: {}", step_name));
                    }
                }
            }
        }

        println!("[trace_id:{}] Flow execution completed with advanced error handling", trace_id);
        Ok(())
    }
}
