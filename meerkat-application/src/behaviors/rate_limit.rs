use std::any::Any;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, PipelineBehavior, PipelineNext};

pub struct RateLimitKey {
    pub key_token: String,
    pub max_per_window: Option<u64>,
}

struct WindowState {
    count: u64,
    window_start: Instant,
}

pub struct RateLimitBehavior {
    counters: DashMap<String, WindowState>,
    max_per_window: u64,
    window_duration: Duration,
}

impl Default for RateLimitBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitBehavior {
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            max_per_window: 1000,
            window_duration: Duration::from_secs(60),
        }
    }

    pub fn with_max_per_window(mut self, n: u64) -> Self {
        self.max_per_window = n;
        self
    }

    pub fn with_window_duration(mut self, duration: Duration) -> Self {
        self.window_duration = duration;
        self
    }
}

#[async_trait]
impl PipelineBehavior<RequestContext, ApplicationError> for RateLimitBehavior {
    async fn handle(
        &self,
        extensions: &Extensions,
        _ctx: &RequestContext,
        next: PipelineNext<'_, ApplicationError>,
    ) -> Result<Box<dyn Any + Send + Sync>, ApplicationError> {
        let Some(key) = extensions.get::<RateLimitKey>() else {
            return next.run().await;
        };

        let effective_limit = key.max_per_window.unwrap_or(self.max_per_window);
        let now = Instant::now();

        {
            let mut entry = self
                .counters
                .entry(key.key_token.clone())
                .or_insert_with(|| WindowState {
                    count: 0,
                    window_start: now,
                });

            let state = entry.value_mut();
            let elapsed = now.duration_since(state.window_start);

            if elapsed >= self.window_duration {
                state.count = 0;
                state.window_start = now;
            }

            state.count += 1;

            if state.count > effective_limit {
                let retry_after = self.window_duration.saturating_sub(elapsed);
                return Err(ApplicationError::RateLimited {
                    retry_after_secs: retry_after.as_secs().max(1),
                });
            }
        }

        next.run().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::context::RequestContext;
    use crate::error::ApplicationError;
    use mediator_rs::{Extensions, Handler, Mediator, Request};

    use super::*;

    struct TestCommand;
    impl Request for TestCommand {
        type Output = String;
    }

    struct RateLimitedCommand {
        key: String,
        max_per_window: Option<u64>,
    }
    impl Request for RateLimitedCommand {
        type Output = String;
        fn extensions(&self) -> Extensions {
            let mut ext = Extensions::new();
            ext.insert(RateLimitKey {
                key_token: self.key.clone(),
                max_per_window: self.max_per_window,
            });
            ext
        }
    }

    struct EchoHandler;

    #[async_trait]
    impl Handler<TestCommand, ApplicationError, RequestContext> for EchoHandler {
        async fn handle(&self, _cmd: TestCommand, _ctx: &RequestContext) -> Result<String, ApplicationError> {
            Ok("ok".to_string())
        }
    }

    #[async_trait]
    impl Handler<RateLimitedCommand, ApplicationError, RequestContext> for EchoHandler {
        async fn handle(&self, _cmd: RateLimitedCommand, _ctx: &RequestContext) -> Result<String, ApplicationError> {
            Ok("ok".to_string())
        }
    }

    fn build_mediator(behavior: RateLimitBehavior) -> Mediator<RequestContext, ApplicationError> {
        let mut mediator = Mediator::new();
        mediator.add_behavior(Arc::new(behavior));
        mediator.register::<TestCommand, _>(EchoHandler);
        mediator.register::<RateLimitedCommand, _>(EchoHandler);
        mediator
    }

    #[tokio::test]
    async fn given_command_without_rate_limit_key_then_passes_through() {
        // arrange
        let mediator = build_mediator(RateLimitBehavior::new().with_max_per_window(1));
        let ctx = RequestContext::test();

        // act
        let result = mediator.dispatch(TestCommand, &ctx).await;

        // assert
        assert_eq!(result.unwrap(), "ok");
    }

    #[tokio::test]
    async fn given_events_within_limit_then_passes_through() {
        // arrange
        let mediator = build_mediator(RateLimitBehavior::new().with_max_per_window(5));
        let ctx = RequestContext::test();

        // act & assert
        for _ in 0..5 {
            let result = mediator
                .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
                .await;
            assert_eq!(result.unwrap(), "ok");
        }
    }

    #[tokio::test]
    async fn given_events_exceeding_limit_then_returns_rate_limited() {
        // arrange
        let mediator = build_mediator(RateLimitBehavior::new().with_max_per_window(3));
        let ctx = RequestContext::test();

        for _ in 0..3 {
            mediator
                .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
                .await
                .unwrap();
        }

        // act
        let result = mediator
            .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
            .await;

        // assert
        match result {
            Err(mediator_rs::MediatorError::HandlerError(ApplicationError::RateLimited { retry_after_secs })) => {
                assert!(retry_after_secs >= 1);
            }
            other => panic!("Expected RateLimited, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn given_different_keys_then_limits_independently() {
        // arrange
        let mediator = build_mediator(RateLimitBehavior::new().with_max_per_window(2));
        let ctx = RequestContext::test();

        for _ in 0..2 {
            mediator
                .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
                .await
                .unwrap();
        }

        // act — key-b should still be within limits
        let result = mediator
            .dispatch(RateLimitedCommand { key: "key-b".into(), max_per_window: None }, &ctx)
            .await;

        // assert
        assert_eq!(result.unwrap(), "ok");

        // act — key-a should be rate limited
        let result = mediator
            .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
            .await;

        // assert
        match result {
            Err(mediator_rs::MediatorError::HandlerError(ApplicationError::RateLimited { .. })) => (),
            other => panic!("Expected RateLimited for key-a, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn given_key_with_custom_limit_then_uses_per_key_limit() {
        // arrange — global default is 1000, but per-key limit is 2
        let mediator = build_mediator(RateLimitBehavior::new().with_max_per_window(1000));
        let ctx = RequestContext::test();

        for _ in 0..2 {
            mediator
                .dispatch(RateLimitedCommand { key: "key-custom".into(), max_per_window: Some(2) }, &ctx)
                .await
                .unwrap();
        }

        // act — third event should be rejected at per-key limit of 2
        let result = mediator
            .dispatch(RateLimitedCommand { key: "key-custom".into(), max_per_window: Some(2) }, &ctx)
            .await;

        // assert
        match result {
            Err(mediator_rs::MediatorError::HandlerError(ApplicationError::RateLimited { .. })) => (),
            other => panic!("Expected RateLimited, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn given_window_expired_then_counter_resets() {
        // arrange
        let mediator = build_mediator(
            RateLimitBehavior::new()
                .with_max_per_window(2)
                .with_window_duration(Duration::from_millis(50)),
        );
        let ctx = RequestContext::test();

        for _ in 0..2 {
            mediator
                .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
                .await
                .unwrap();
        }

        // act — wait for window to expire
        tokio::time::sleep(Duration::from_millis(60)).await;

        let result = mediator
            .dispatch(RateLimitedCommand { key: "key-a".into(), max_per_window: None }, &ctx)
            .await;

        // assert
        assert_eq!(result.unwrap(), "ok");
    }
}
