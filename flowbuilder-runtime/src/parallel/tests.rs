#[cfg(test)]
mod tests {
    use crate::{ParallelConfig, ParallelExecutor, ParallelResults};
    use std::time::Duration;
    use flowbuilder_context::FlowContext;

    #[tokio::test]
    async fn test_parallel_config_creation() {
        let config = ParallelConfig::default();
        assert!(config.max_concurrency.is_none());
        assert!(config.step_timeout.is_none());
        assert!(!config.fail_fast);
        assert!(config.batch_size.is_none());

        let config = ParallelConfig::with_max_concurrency(5)
            .timeout(Duration::from_secs(10))
            .with_fail_fast()
            .batch_size(3);

        assert_eq!(config.max_concurrency, Some(5));
        assert_eq!(config.step_timeout, Some(Duration::from_secs(10)));
        assert!(config.fail_fast);
        assert_eq!(config.batch_size, Some(3));
    }

    #[tokio::test]
    async fn test_parallel_executor_creation() {
        let executor = ParallelExecutor::new();
        assert_eq!(executor.config.max_concurrency, None);

        let config = ParallelConfig::with_max_concurrency(3);
        let executor = ParallelExecutor::with_config(config);
        assert_eq!(executor.config.max_concurrency, Some(3));
    }

    #[tokio::test]
    async fn test_empty_steps_execution() {
        let executor = ParallelExecutor::new();
        let context = std::sync::Arc::new(tokio::sync::Mutex::new(FlowContext::default()));

        let results = executor.execute_parallel_detailed(vec![], context).await.unwrap();

        assert_eq!(results.success_count, 0);
        assert_eq!(results.failed_count, 0);
        assert!(results.errors.is_empty());
    }

    #[tokio::test]
    async fn test_parallel_results_display() {
        let results = ParallelResults {
            success_count: 3,
            failed_count: 1,
            errors: vec![anyhow::anyhow!("Test error")],
            total_duration: Duration::from_millis(100),
        };

        assert_eq!(results.success_count, 3);
        assert_eq!(results.failed_count, 1);
        assert_eq!(results.errors.len(), 1);
        assert_eq!(results.total_duration, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_config_chaining() {
        let config = ParallelConfig::default()
            .max_concurrency(5)
            .timeout(Duration::from_secs(30))
            .with_fail_fast()
            .batch_size(10);

        assert_eq!(config.max_concurrency, Some(5));
        assert_eq!(config.step_timeout, Some(Duration::from_secs(30)));
        assert!(config.fail_fast);
        assert_eq!(config.batch_size, Some(10));
    }

    #[tokio::test]
    async fn test_convenience_constructors() {
        let config1 = ParallelConfig::fail_fast();
        assert!(config1.fail_fast);

        let config2 = ParallelConfig::batched(5);
        assert_eq!(config2.batch_size, Some(5));

        let config3 = ParallelConfig::with_timeout(Duration::from_secs(60));
        assert_eq!(config3.step_timeout, Some(Duration::from_secs(60)));
    }
}
