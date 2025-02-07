use futures_util::{FutureExt, Stream};
use pin_project_lite::pin_project;
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::{select, task::JoinSet};

type UnwindResult<T> = Result<T, Box<dyn std::any::Any + Send>>;

pub fn worker_map<F, Fut, T, U>(
    func: F,
    workers: NonZeroUsize,
    buffer_size: NonZeroUsize,
) -> (Sender<T>, Receiver<T, U>)
where
    F: Fn(T) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = U> + Send,
    T: Send + 'static,
    U: Send + 'static,
{
    let (input_sender, input_receiver) = async_channel::bounded(buffer_size.get());
    let (output_sender, output_receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut tasks = JoinSet::new();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    for _ in 0..workers.get() {
        tasks.spawn({
            let func = func.clone();
            let input = input_receiver.clone();
            let output = output_sender.clone();
            let mut done = shutdown_receiver.clone();
            async move {
                loop {
                    let work = select! {
                        _ = done.changed() => break,
                        r = input.recv() => match r {
                            Ok(work) => work,
                            Err(_) => break,
                        },
                    };
                    let r = std::panic::AssertUnwindSafe(func(work))
                        .catch_unwind()
                        .await;
                    if output.send(r).is_err() {
                        break;
                    }
                }
            }
        });
    }
    (
        Sender(input_sender),
        Receiver {
            inner: output_receiver,
            closer: Closer(input_receiver),
            shutdown_sender,
            _tasks: tasks,
        },
    )
}

#[derive(Debug)]
pub struct Sender<T>(async_channel::Sender<T>);

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> async_channel::Send<'_, T> {
        self.0.send(msg)
    }

    pub fn try_send(&self, msg: T) -> Result<(), async_channel::TrySendError<T>> {
        self.0.try_send(msg)
    }

    pub fn close(&self) -> bool {
        self.0.close()
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        match self.0.capacity() {
            Some(n) => n,
            None => unreachable!("channel should be bounded"),
        }
    }
}

// Clone can't be derived, as that would erroneously add `T: Clone` bounds to
// the impl.
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Sender(self.0.clone())
    }
}

// pin_project lets us call poll_recv() in poll_next() without even calling
// project().  Not sure how.
pin_project! {
    #[derive(Debug)]
    pub struct Receiver<T, U> {
        inner: tokio::sync::mpsc::UnboundedReceiver<UnwindResult<U>>,
        closer: Closer<T>,
        shutdown_sender: tokio::sync::watch::Sender<bool>,
        _tasks: JoinSet<()>,
    }
}

impl<T: Send, U: Send> Receiver<T, U> {
    pub async fn recv(&mut self) -> Option<U> {
        match self.inner.recv().await? {
            Ok(r) => Some(r),
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}

impl<T, U> Receiver<T, U> {
    pub fn close(&self) -> bool {
        self.closer.close()
    }

    pub fn is_closed(&self) -> bool {
        self.closer.is_closed()
    }

    // Returns `true` if `shutdown()` was not already called before
    pub fn shutdown(&self) -> bool {
        self.close();
        self.shutdown_sender.send_if_modified(|b| {
            if !*b {
                *b = true;
                true
            } else {
                false
            }
        })
    }

    pub fn is_shutdown(&self) -> bool {
        *self.shutdown_sender.borrow()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<U>> {
        match ready!(self.inner.poll_recv(cx)) {
            Some(Ok(r)) => Some(r).into(),
            Some(Err(e)) => std::panic::resume_unwind(e),
            None => None.into(),
        }
    }
}

impl<T, U: 'static> Stream for Receiver<T, U> {
    type Item = U;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<U>> {
        self.poll_recv(cx)
    }
}

// This type is needed because putting the Drop impl on Receiver instead
// conflicts with pin_project_lite.
#[derive(Debug)]
struct Closer<T>(async_channel::Receiver<T>);

impl<T> Closer<T> {
    fn close(&self) -> bool {
        self.0.close()
    }

    fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

impl<T> Drop for Closer<T> {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn collect() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, receiver) = worker_map(|n| async move { n + 1 }, workers, workers);
        for i in 0..20 {
            sender.send(i).await.unwrap();
        }
        assert!(!receiver.is_closed());
        drop(sender);
        assert!(receiver.is_closed());
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (1..21).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn reraise_panic_recv() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, mut receiver) = worker_map(
            |n: u32| async move {
                if n < 4 {
                    n + 1
                } else {
                    panic!("I can't count that high!")
                }
            },
            workers,
            workers,
        );
        for i in 0..20 {
            sender.send(i).await.unwrap();
        }
        drop(sender);
        let mut outputs = Vec::new();
        let mut panics = 0;
        loop {
            let r = std::panic::AssertUnwindSafe(receiver.recv())
                .catch_unwind()
                .await;
            match r {
                Ok(Some(n)) => outputs.push(n),
                Ok(None) => break,
                Err(_) => panics += 1,
            }
        }
        assert_eq!(panics, 16);
        outputs.sort_unstable();
        assert_eq!(outputs, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn reraise_panic_next() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, mut receiver) = worker_map(
            |n: u32| async move {
                if n < 4 {
                    n + 1
                } else {
                    panic!("I can't count that high!")
                }
            },
            workers,
            workers,
        );
        for i in 0..20 {
            sender.send(i).await.unwrap();
        }
        drop(sender);
        let mut outputs = Vec::new();
        let mut panics = 0;
        loop {
            let r = std::panic::AssertUnwindSafe(receiver.next())
                .catch_unwind()
                .await;
            match r {
                Ok(Some(n)) => outputs.push(n),
                Ok(None) => break,
                Err(_) => panics += 1,
            }
        }
        assert_eq!(panics, 16);
        outputs.sort_unstable();
        assert_eq!(outputs, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn close_receiver() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, receiver) = worker_map(|n| async move { n + 1 }, workers, workers);
        for i in 0..5 {
            sender.send(i).await.unwrap();
        }
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(receiver.close());
        assert!(sender.send(5).await.is_err());
        assert!(!receiver.is_shutdown());
        assert!(receiver.is_closed());
        assert!(sender.is_closed());
        drop(sender);
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (1..6).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn close_sender() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, receiver) = worker_map(|n| async move { n + 1 }, workers, workers);
        for i in 0..5 {
            sender.send(i).await.unwrap();
        }
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(sender.close());
        assert!(sender.send(5).await.is_err());
        assert!(!receiver.is_shutdown());
        assert!(receiver.is_closed());
        assert!(sender.is_closed());
        drop(sender);
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (1..6).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn close_on_shutdown() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, receiver) = worker_map(|n| async move { n + 1 }, workers, workers);
        for i in 0..5 {
            sender.send(i).await.unwrap();
        }
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(receiver.shutdown());
        assert!(sender.send(5).await.is_err());
        assert!(receiver.is_shutdown());
        assert!(receiver.is_closed());
        assert!(sender.is_closed());
        drop(sender);
        // Note that, because shutdown() prevents queued tasks from running,
        // the receiver will nondeterministically return a subset of the
        // incremented inputs.
        assert!(receiver.all(|n| async move { (1..6).contains(&n) }).await);
    }
}
