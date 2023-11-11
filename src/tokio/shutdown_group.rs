use futures::stream::{FuturesUnordered, StreamExt};
use std::future::Future;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
pub struct ShutdownGroup {
    handles: FuturesUnordered<JoinHandle<()>>,
    token: CancellationToken,
}

impl ShutdownGroup {
    pub fn new() -> Self {
        ShutdownGroup {
            handles: FuturesUnordered::new(),
            token: CancellationToken::new(),
        }
    }

    pub fn spawn<F, Fut>(&self, func: F)
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let future = func(self.token.clone());
        self.handles.push(tokio::spawn(future));
    }

    async fn join(&mut self) {
        while self.handles.next().await.is_some() {}
    }

    pub async fn shutdown(mut self, duration: Duration) {
        if timeout(duration, self.join()).await.is_err() {
            self.token.cancel();
        }
        self.join().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_shutdown_group() {
        let group = ShutdownGroup::new();
        let task1_finished = Arc::new(AtomicBool::new(false));
        let my_finished = task1_finished.clone();
        group.spawn(|token| async move {
            tokio::select! {
                _ = token.cancelled() => (),
                _ = futures::future::ready(()) => my_finished.store(true, Ordering::Release),
            }
        });
        let task2_cancelled = Arc::new(AtomicBool::new(false));
        let my_cancelled = task2_cancelled.clone();
        group.spawn(|token| async move {
            tokio::select! {
                _ = token.cancelled() => my_cancelled.store(true, Ordering::Release),
                _ = tokio::time::sleep(Duration::from_secs(10)) => (),
            }
        });
        group.shutdown(Duration::from_secs(1)).await;
        assert!(task1_finished.load(Ordering::Acquire));
        assert!(task2_cancelled.load(Ordering::Acquire));
    }
}
