use std::time::Duration;
use tokio::time::sleep;
use tracing;

pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                tracing::warn!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1,
                    e,
                    delay
                );
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}
