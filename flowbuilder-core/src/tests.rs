#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_basic_flow() {
        let flow = FlowBuilder::new()
            .step(|_ctx| async move { Ok(()) })
            .build();

        let result = flow.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_named_step() {
        let flow = FlowBuilder::new()
            .named_step("test_step", |_ctx| async move { Ok(()) })
            .build();

        let result = flow.execute().await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.step_logs.len(), 1);
        assert_eq!(context.step_logs[0].step_name, "test_step");
    }

    #[tokio::test]
    async fn test_conditional_step() {
        let flow = FlowBuilder::new()
            .step_if(|_ctx| true, |_ctx| async move { Ok(()) })
            .build();

        let result = flow.execute().await;
        assert!(result.is_ok());
    }
}
