use anyhow::Result;
use flowbuilder_context::SharedContext;
use flowbuilder_core::Step;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;

#[cfg(test)]
mod tests;

/// A wrapper around Step to make it easier to work with in parallel contexts
#[derive(Clone)]
pub struct ParallelStep {
    pub name: String,
    pub step_fn: Arc<
        dyn Fn(SharedContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync,
    >,
}

impl ParallelStep {
    pub fn new<F, Fut>(name: impl Into<String>, f: F) -> Self
    where
        F: Fn(SharedContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            step_fn: Arc::new(move |ctx| Box::pin(f(ctx))),
        }
    }

    pub async fn execute(&self, context: SharedContext) -> Result<()> {
        (self.step_fn)(context).await
    }
}

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent steps (None for unlimited)
    pub max_concurrency: Option<usize>,
    /// Timeout for individual steps
    pub step_timeout: Option<Duration>,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Batch size for batched execution
    pub batch_size: Option<usize>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrency: None,
            step_timeout: None,
            fail_fast: false,
            batch_size: None,
        }
    }
}

impl ParallelConfig {
    /// Create a config with maximum concurrency limit
    pub fn with_max_concurrency(concurrency: usize) -> Self {
        Self {
            max_concurrency: Some(concurrency),
            ..Default::default()
        }
    }

    /// Create a config with step timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            step_timeout: Some(timeout),
            ..Default::default()
        }
    }

    /// Create a config that fails fast on first error
    pub fn fail_fast() -> Self {
        Self {
            fail_fast: true,
            ..Default::default()
        }
    }

    /// Create a config for batched execution
    pub fn batched(batch_size: usize) -> Self {
        Self {
            batch_size: Some(batch_size),
            ..Default::default()
        }
    }

    /// Set maximum concurrency
    pub fn max_concurrency(mut self, concurrency: usize) -> Self {
        self.max_concurrency = Some(concurrency);
        self
    }

    /// Set step timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.step_timeout = Some(timeout);
        self
    }

    /// Enable fail fast mode
    pub fn with_fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }
}

/// Results from parallel execution
#[derive(Debug)]
pub struct ParallelResults {
    /// Number of successful steps
    pub success_count: usize,
    /// Number of failed steps
    pub failed_count: usize,
    /// Errors that occurred
    pub errors: Vec<anyhow::Error>,
    /// Total execution time
    pub total_duration: Duration,
}

/// Executor for parallel step execution
pub struct ParallelExecutor {
    config: ParallelConfig,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            config: ParallelConfig::default(),
        }
    }

    /// Create executor with custom configuration
    pub fn with_config(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Execute steps in parallel with default configuration
    pub async fn execute_parallel(&self, steps: Vec<Step>, context: SharedContext) -> Result<()> {
        let results = self.execute_parallel_detailed(steps, context).await?;

        if !results.errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Parallel execution failed: {} errors occurred",
                results.errors.len()
            ));
        }

        Ok(())
    }

    /// Execute steps in parallel and return detailed results
    pub async fn execute_parallel_detailed(
        &self,
        steps: Vec<Step>,
        context: SharedContext,
    ) -> Result<ParallelResults> {
        let start_time = std::time::Instant::now();

        if steps.is_empty() {
            return Ok(ParallelResults {
                success_count: 0,
                failed_count: 0,
                errors: Vec::new(),
                total_duration: start_time.elapsed(),
            });
        }

        // If batch execution is configured
        if let Some(batch_size) = self.config.batch_size {
            return self
                .execute_batched_detailed(steps, context, batch_size)
                .await;
        }

        let results = if let Some(max_concurrency) = self.config.max_concurrency {
            self.execute_with_semaphore(steps, context, max_concurrency)
                .await
        } else {
            self.execute_unlimited(steps, context).await
        };

        let total_duration = start_time.elapsed();

        match results {
            Ok((success_count, errors)) => Ok(ParallelResults {
                success_count,
                failed_count: errors.len(),
                errors,
                total_duration,
            }),
            Err(e) => Err(e),
        }
    }

    /// Execute with unlimited concurrency
    async fn execute_unlimited(
        &self,
        steps: Vec<Step>,
        context: SharedContext,
    ) -> Result<(usize, Vec<anyhow::Error>)> {
        let mut join_set = JoinSet::new();

        for (index, step) in steps.into_iter().enumerate() {
            let ctx = context.clone();
            let step_timeout = self.config.step_timeout;

            join_set.spawn(async move {
                let step_result = if let Some(timeout_duration) = step_timeout {
                    match timeout(timeout_duration, step(ctx.clone())).await {
                        Ok(result) => result,
                        Err(_) => {
                            // Mark step as timed out in context
                            let mut guard = ctx.lock().await;
                            guard.end_step_timeout(&format!("parallel_step_{}", index));
                            Err(anyhow::anyhow!("Step {} timed out", index))
                        }
                    }
                } else {
                    step(ctx).await
                };

                (index, step_result)
            });
        }

        let mut errors = Vec::new();
        let mut success_count = 0;

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((index, step_result)) => {
                    match step_result {
                        Ok(()) => success_count += 1,
                        Err(e) => {
                            errors.push(anyhow::anyhow!("Step {}: {}", index, e));

                            if self.config.fail_fast {
                                // Abort remaining tasks
                                join_set.abort_all();
                                return Err(anyhow::anyhow!("Fail-fast enabled: {}", e));
                            }
                        }
                    }
                }
                Err(join_error) => {
                    errors.push(anyhow::anyhow!("Join error: {}", join_error));
                }
            }
        }

        Ok((success_count, errors))
    }

    /// Execute with concurrency limit using semaphore
    async fn execute_with_semaphore(
        &self,
        steps: Vec<Step>,
        context: SharedContext,
        max_concurrency: usize,
    ) -> Result<(usize, Vec<anyhow::Error>)> {
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut join_set = JoinSet::new();

        for (index, step) in steps.into_iter().enumerate() {
            let ctx = context.clone();
            let sem = semaphore.clone();
            let step_timeout = self.config.step_timeout;

            join_set.spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore permit: {}", e))?;

                let step_result = if let Some(timeout_duration) = step_timeout {
                    match timeout(timeout_duration, step(ctx.clone())).await {
                        Ok(result) => result,
                        Err(_) => {
                            let mut guard = ctx.lock().await;
                            guard.end_step_timeout(&format!("parallel_step_{}", index));
                            Err(anyhow::anyhow!("Step {} timed out", index))
                        }
                    }
                } else {
                    step(ctx).await
                };

                Ok((index, step_result))
            });
        }

        let mut errors = Vec::new();
        let mut success_count = 0;

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(task_result) => match task_result {
                    Ok((index, step_result)) => match step_result {
                        Ok(()) => success_count += 1,
                        Err(e) => {
                            errors.push(anyhow::anyhow!("Step {}: {}", index, e));

                            if self.config.fail_fast {
                                join_set.abort_all();
                                return Err(anyhow::anyhow!("Fail-fast enabled: {}", e));
                            }
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                    }
                },
                Err(join_error) => {
                    errors.push(anyhow::anyhow!("Join error: {}", join_error));
                }
            }
        }

        Ok((success_count, errors))
    }

    /// Execute steps in batches with limited concurrency
    pub async fn execute_batched(
        &self,
        steps: Vec<Step>,
        context: SharedContext,
        batch_size: usize,
    ) -> Result<()> {
        let results = self
            .execute_batched_detailed(steps, context, batch_size)
            .await?;

        if !results.errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Batched execution failed: {} errors occurred",
                results.errors.len()
            ));
        }

        Ok(())
    }

    /// Execute steps in batches and return detailed results
    async fn execute_batched_detailed(
        &self,
        mut steps: Vec<Step>,
        context: SharedContext,
        batch_size: usize,
    ) -> Result<ParallelResults> {
        let start_time = std::time::Instant::now();
        let mut total_success = 0;
        let mut total_errors = Vec::new();

        // Process steps in batches
        let mut step_index = 0;
        while !steps.is_empty() {
            let batch_size = batch_size.min(steps.len());
            let batch: Vec<Step> = steps.drain(0..batch_size).collect();

            let batch_start = std::time::Instant::now();
            println!("Processing batch of {} steps...", batch.len());

            // Execute each step in the batch sequentially
            for step in batch {
                let ctx = context.clone();
                match step(ctx).await {
                    Ok(()) => {
                        total_success += 1;
                        println!("  ✓ Step {}: Success", step_index);
                    }
                    Err(e) => {
                        total_errors.push(anyhow::anyhow!("Step {}: {}", step_index, e));
                        println!("  ✗ Step {}: Failed - {}", step_index, e);

                        if self.config.fail_fast {
                            return Err(anyhow::anyhow!(
                                "Fail-fast enabled in batch execution: {}",
                                e
                            ));
                        }
                    }
                }
                step_index += 1;
            }

            let batch_duration = batch_start.elapsed();
            println!("  Batch completed in {:?}", batch_duration);

            // Small delay between batches to prevent overwhelming
            if let Some(delay) = self.config.step_timeout {
                let inter_batch_delay = delay.min(Duration::from_millis(100));
                tokio::time::sleep(inter_batch_delay).await;
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(ParallelResults {
            success_count: total_success,
            failed_count: total_errors.len(),
            errors: total_errors,
            total_duration: start_time.elapsed(),
        })
    }

    /// Execute steps with better error tracking and reporting
    pub async fn execute_with_monitoring(
        &self,
        steps: Vec<ParallelStep>,
        context: SharedContext,
    ) -> Result<ParallelResults> {
        let start_time = std::time::Instant::now();

        if steps.is_empty() {
            return Ok(ParallelResults {
                success_count: 0,
                failed_count: 0,
                errors: Vec::new(),
                total_duration: start_time.elapsed(),
            });
        }

        let mut join_set = JoinSet::new();
        let semaphore = if let Some(max_concurrency) = self.config.max_concurrency {
            Some(Arc::new(Semaphore::new(max_concurrency)))
        } else {
            None
        };

        for (index, parallel_step) in steps.into_iter().enumerate() {
            let ctx = context.clone();
            let step_timeout = self.config.step_timeout;
            let sem = semaphore.clone();
            let step_name = parallel_step.name.clone();

            join_set.spawn(async move {
                let _permit = if let Some(ref semaphore) = sem {
                    match semaphore.acquire().await {
                        Ok(permit) => Some(permit),
                        Err(e) => {
                            return (
                                index,
                                step_name,
                                Err(anyhow::anyhow!("Failed to acquire semaphore permit: {}", e)),
                            );
                        }
                    }
                } else {
                    None
                };

                println!("Starting step {}: {}", index, step_name);

                let step_result = if let Some(timeout_duration) = step_timeout {
                    match timeout(timeout_duration, parallel_step.execute(ctx.clone())).await {
                        Ok(result) => result,
                        Err(_) => {
                            println!("Step {} ({}) timed out", index, step_name);
                            Err(anyhow::anyhow!("Step {} ({}) timed out", index, step_name))
                        }
                    }
                } else {
                    parallel_step.execute(ctx).await
                };

                (index, step_name, step_result)
            });
        }

        let mut errors = Vec::new();
        let mut success_count = 0;

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((index, step_name, step_result)) => match step_result {
                    Ok(()) => {
                        success_count += 1;
                        println!("✓ Step {} ({}): Success", index, step_name);
                    }
                    Err(e) => {
                        errors.push(anyhow::anyhow!("Step {} ({}): {}", index, step_name, e));
                        println!("✗ Step {} ({}): Failed - {}", index, step_name, e);

                        if self.config.fail_fast {
                            join_set.abort_all();
                            return Err(anyhow::anyhow!(
                                "Fail-fast enabled: Step {} ({}) failed: {}",
                                index,
                                step_name,
                                e
                            ));
                        }
                    }
                },
                Err(join_error) => {
                    errors.push(anyhow::anyhow!("Join error: {}", join_error));
                }
            }
        }

        Ok(ParallelResults {
            success_count,
            failed_count: errors.len(),
            errors,
            total_duration: start_time.elapsed(),
        })
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}
