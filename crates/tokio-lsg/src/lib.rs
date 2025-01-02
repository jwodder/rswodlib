use futures_util::Stream;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, PoisonError};
use std::task::{Context, Poll};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Semaphore,
};
use tokio_util::sync::CancellationToken;

/// A task group with the following properties:
///
/// - No more than a certain number of tasks are ever active at once.
///
/// - Each task is passed a `CancellationToken` that can be used for graceful
///   shutdown.
///
/// - `LimitedShutdownGroup<T>` is a `Stream` of the return values of the tasks
///   (which must all be `T`).
///
/// - `shutdown()` cancels the cancellation token and prevents any further
///   pending tasks from running.
#[derive(Debug)]
pub struct LimitedShutdownGroup<T> {
    sender: Mutex<Option<Sender<T>>>,
    receiver: Receiver<T>,
    semaphore: Arc<Semaphore>,
    token: CancellationToken,
}

impl<T: Send + 'static> LimitedShutdownGroup<T> {
    pub fn new(limit: usize) -> Self {
        let (sender, receiver) = channel(limit);
        LimitedShutdownGroup {
            sender: Mutex::new(Some(sender)),
            receiver,
            semaphore: Arc::new(Semaphore::new(limit)),
            token: CancellationToken::new(),
        }
    }

    pub fn spawn<F, Fut>(&self, func: F)
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future<Output = T> + Send + 'static,
    {
        let sender = {
            let s = self.sender.lock().unwrap_or_else(PoisonError::into_inner);
            (*s).clone()
        };
        if let Some(sender) = sender {
            let future = func(self.token.clone());
            let sem = Arc::clone(&self.semaphore);
            tokio::spawn(async move {
                if let Ok(_permit) = sem.acquire().await {
                    let _ = sender.send(future.await).await;
                }
            });
        }
    }

    pub fn close(&self) {
        let mut s = self.sender.lock().unwrap_or_else(PoisonError::into_inner);
        *s = None;
    }

    pub fn shutdown(&self) {
        self.close();
        self.semaphore.close();
        self.token.cancel();
    }
}

impl<T: Send + 'static> Stream for LimitedShutdownGroup<T> {
    type Item = T;

    /// Poll for one of the tasks in the group to complete and return its
    /// return value.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
