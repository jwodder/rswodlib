use futures_util::{FutureExt, Stream};
use pin_project_lite::pin_project;
use std::fmt;
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::task::{ready, Context, Poll};
use tokio::{sync::mpsc, task::JoinSet};

type UnwindResult<T> = Result<T, Box<dyn std::any::Any + Send>>;

/// Map values through asynchronous worker tasks.
///
/// `worker_map()` spawns `workers` concurrent tasks that loop over the values
/// sent to the sender, apply `func` to them, and send the results to the
/// receiver.  The sender is clonable, but the receiver is not.  If any
/// application of `func` panics, the panic is reraised by the receiver.  Once
/// the sender and all of its clones have been dropped, and once the results of
/// all function applications have been received, the receiver stream will
/// close.  If the receiver is dropped, the worker tasks are aborted.
///
/// `buffer_size` is the capacity of the the sender channel.  The receiver
/// channel is unbounded.
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
    let (output_sender, output_receiver) = mpsc::unbounded_channel();
    let mut tasks = JoinSet::new();
    let done = Arc::new(AtomicBool::new(false));
    for _ in 0..workers.get() {
        tasks.spawn({
            let func = func.clone();
            let input = input_receiver.clone();
            let output = output_sender.clone();
            let done = done.clone();
            async move {
                while let Ok(work) = input.recv().await {
                    if done.load(Ordering::SeqCst) {
                        break;
                    }
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
        Sender {
            inner: input_sender,
            done: done.clone(),
        },
        Receiver {
            inner: output_receiver,
            closer: Closer(input_receiver),
            done,
            _tasks: tasks,
        },
    )
}

/// A handle for sending values to a [`worker_map()`] task.
///
/// Senders can be cloned and shared among threads. When all senders associated
/// with a sender-receiver pair are dropped, the receiver becomes closed.
#[derive(Debug)]
pub struct Sender<T> {
    inner: async_channel::Sender<T>,
    done: Arc<AtomicBool>,
}

impl<T> Sender<T> {
    /// Sends a task input into the channel, queuing it to be processed by the
    /// next free worker task.
    ///
    /// If the channel is full, this method waits until there is space.
    ///
    /// If the channel is closed, this method returns an error.
    pub fn send(&self, msg: T) -> async_channel::Send<'_, T> {
        self.inner.send(msg)
    }

    /// Attempts to send a message into the channel.
    ///
    /// If the channel is full or closed, this method returns an error.
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(msg).map_err(Into::into)
    }

    /// Closes the channel.  Any further calls to [`send()`](Self::send) or
    /// [`try_send()`](Self::try_send) will return an error.
    ///
    /// Returns `true` if this call has closed the channel and it was not
    /// closed already.
    ///
    /// Any pending task inputs will still be processed after calling
    /// `close()`.
    pub fn close(&self) -> bool {
        self.inner.close()
    }

    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Calls [`close()`](Self::close) and additionally instructs the worker
    /// tasks to not process any pending task inputs.  Any inputs currently
    /// being processed are still processed to completion.
    ///
    /// Returns `true` if this call has shut down the sender/receiver and it
    /// was not shut down already.
    pub fn shutdown(&self) -> bool {
        self.close();
        !self.done.swap(true, Ordering::SeqCst)
    }

    /// Returns `true` if the sender/receiver is shut down.
    pub fn is_shutdown(&self) -> bool {
        self.done.load(Ordering::SeqCst)
    }

    /// Returns `true` if the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if the channel is full.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Returns the number of pending inputs in the channel.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the channel capacity.
    pub fn capacity(&self) -> usize {
        match self.inner.capacity() {
            Some(n) => n,
            None => unreachable!("channel should be bounded"),
        }
    }
}

// Clone can't be derived, as that would erroneously add `T: Clone` bounds to
// the impl.
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Sender {
            inner: self.inner.clone(),
            done: self.done.clone(),
        }
    }
}

// pin_project! lets us call poll_recv() in poll_next() without even calling
// project().  Not sure how.
pin_project! {
    /// A handle for receiving the results of operations from
    /// [`worker_map()`] tasks.
    ///
    /// Receivers cannot be cloned.
    ///
    /// When a receiver is dropped, all corresponding worker tasks are aborted,
    /// and the sender channels are closed.
    #[derive(Debug)]
    pub struct Receiver<T, U> {
        inner: mpsc::UnboundedReceiver<UnwindResult<U>>,
        closer: Closer<T>,
        done: Arc<AtomicBool>,
        // The JoinSet of tasks is kept around so that the tasks are aborted
        // only when the Receiver is dropped.
        _tasks: JoinSet<()>,
    }
}

impl<T: Send, U: Send> Receiver<T, U> {
    /// Receives a result from a worker task.
    ///
    /// If the channel is empty, this method waits until there is a message.
    ///
    /// If the channel is closed (due to either all senders having been dropped
    /// or `close()` having been called), this method receives a result or
    /// returns `None` if there are no more results.
    ///
    /// # Panics
    ///
    /// If the channel receives a result from a task that panicked, this method
    /// resumes unwinding the panic.
    pub async fn recv(&mut self) -> Option<U> {
        match self.inner.recv().await? {
            Ok(r) => Some(r),
            Err(e) => std::panic::resume_unwind(e),
        }
    }

    /// Tries to receive the next result for this receiver.
    ///
    /// This method returns the [`Empty`] error if the channel is currently
    /// empty but there are still outstanding [senders].
    ///
    /// This method returns the [`Done`] error if the channel is currently
    /// empty and there are no outstanding [senders].
    ///
    /// Unlike the [`poll_recv`] method, this method will never return an
    /// [`Empty`] error spuriously.
    ///
    /// [`Empty`]: TryRecvError::Empty
    /// [`Done`]: TryRecvError::Done
    /// [`poll_recv`]: Self::poll_recv
    /// [senders]: Sender
    ///
    /// # Panics
    ///
    /// If the channel receives a result from a task that panicked, this method
    /// resumes unwinding the panic.
    pub fn try_recv(&mut self) -> Result<U, TryRecvError> {
        match self.inner.try_recv()? {
            Ok(r) => Ok(r),
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}

impl<T, U> Receiver<T, U> {
    /// Closes the corresponding [`Sender`]s.  Any further calls to
    /// [`Sender::send()`] or [`Sender::try_send()`] will return an error.
    ///
    /// Returns `true` if this call has closed the senders and they were not
    /// closed already.
    ///
    /// Any pending task inputs will still be processed after calling
    /// `close()`.
    pub fn close(&self) -> bool {
        self.closer.close()
    }

    /// Returns `true` if the sender/receiver is closed.
    pub fn is_closed(&self) -> bool {
        self.closer.is_closed()
    }

    /// Calls [`close()`](Self::close) and additionally instructs the worker
    /// tasks to not process any pending task inputs.  Any inputs currently
    /// being processed are still processed to completion.
    ///
    /// Returns `true` if this call has shut down the sender/receiver and it
    /// was not shut down already.
    pub fn shutdown(&self) -> bool {
        self.close();
        !self.done.swap(true, Ordering::SeqCst)
    }

    /// Returns `true` if the sender/receiver is shut down.
    pub fn is_shutdown(&self) -> bool {
        self.done.load(Ordering::SeqCst)
    }

    /// Returns `true` if the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of pending outputs in the channel.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Polls to receive the next result on this channel.
    ///
    /// This method returns:
    ///
    ///  * `Poll::Pending` if no results are available but the channel is not
    ///    closed, or if a spurious failure happens.
    ///  * `Poll::Ready(Some(message))` if a result is available.
    ///  * `Poll::Ready(None)` if the channel has been closed and all results
    ///    sent before it was closed have been received.
    ///
    /// # Panics
    ///
    /// If the channel receives a result from a task that panicked, this method
    /// resumes unwinding the panic.
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

    /// Poll for one of the worker tasks to finish processing an input value,
    /// and return the output.
    ///
    /// # Panics
    ///
    /// If the channel receives a result from a task that panicked, this method
    /// resumes unwinding the panic.
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrySendError::Full(_) => write!(f, "Full(..)"),
            TrySendError::Closed(_) => write!(f, "Closed(..)"),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrySendError::Full(_) => write!(f, "cannot send into a full channel"),
            TrySendError::Closed(_) => write!(f, "cannot send into a closed channel"),
        }
    }
}

impl<T> std::error::Error for TrySendError<T> {}

impl<T> From<async_channel::TrySendError<T>> for TrySendError<T> {
    fn from(value: async_channel::TrySendError<T>) -> TrySendError<T> {
        match value {
            async_channel::TrySendError::Full(t) => TrySendError::Full(t),
            async_channel::TrySendError::Closed(t) => TrySendError::Closed(t),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TryRecvError {
    Empty,
    Done,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "output stream is empty"),
            TryRecvError::Done => write!(f, "workers are done"),
        }
    }
}

impl std::error::Error for TryRecvError {}

impl From<mpsc::error::TryRecvError> for TryRecvError {
    fn from(e: mpsc::error::TryRecvError) -> TryRecvError {
        match e {
            mpsc::error::TryRecvError::Empty => TryRecvError::Empty,
            mpsc::error::TryRecvError::Disconnected => TryRecvError::Done,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use tokio::sync::oneshot;

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
        assert!(!sender.is_shutdown());
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(receiver.close());
        assert!(sender.send(5).await.is_err());
        assert!(!sender.is_shutdown());
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
        assert!(!sender.is_shutdown());
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(sender.close());
        assert!(sender.send(5).await.is_err());
        assert!(!sender.is_shutdown());
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
        assert!(!sender.is_shutdown());
        assert!(!receiver.is_shutdown());
        assert!(!receiver.is_closed());
        assert!(!sender.is_closed());
        assert!(receiver.shutdown());
        assert!(sender.send(5).await.is_err());
        assert!(sender.is_shutdown());
        assert!(receiver.is_shutdown());
        assert!(receiver.is_closed());
        assert!(sender.is_closed());
        assert!(!sender.shutdown());
        assert!(!receiver.shutdown());
        assert!(sender.is_shutdown());
        assert!(receiver.is_shutdown());
        assert!(receiver.is_closed());
        assert!(sender.is_closed());
        drop(sender);
        // Note that, because shutdown() prevents queued tasks from running,
        // the receiver will nondeterministically return a subset of the
        // incremented inputs.
        assert!(receiver.all(|n| async move { (1..6).contains(&n) }).await);
    }

    #[tokio::test]
    async fn dropping_receiver_closes_sender() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, receiver) = worker_map(|n| async move { n + 1 }, workers, workers);
        assert!(!sender.is_closed());
        drop(receiver);
        assert!(sender.is_closed());
        assert!(sender.send(5).await.is_err());
    }

    #[tokio::test]
    async fn queued_run_after_close() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, mut receiver) = worker_map(
            |rx: oneshot::Receiver<usize>| async move { rx.await.unwrap() },
            workers,
            workers,
        );
        let mut txes = Vec::new();
        for _ in 0..10 {
            let (tx, rx) = oneshot::channel();
            sender.send(rx).await.unwrap();
            txes.push(tx);
        }
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
        sender.close();
        for (i, tx) in txes.into_iter().enumerate() {
            tx.send(i).unwrap();
        }
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (0..10).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn queued_not_run_after_recv_shutdown() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, mut receiver) = worker_map(
            |rx: oneshot::Receiver<usize>| async move { rx.await.unwrap() },
            workers,
            workers,
        );
        let mut txes = Vec::new();
        for _ in 0..10 {
            let (tx, rx) = oneshot::channel();
            sender.send(rx).await.unwrap();
            txes.push(tx);
        }
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
        receiver.shutdown();
        for (i, tx) in txes.into_iter().enumerate() {
            let _ = tx.send(i);
        }
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (0..5).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn queued_not_run_after_send_shutdown() {
        let workers = NonZeroUsize::new(5).unwrap();
        let (sender, mut receiver) = worker_map(
            |rx: oneshot::Receiver<usize>| async move { rx.await.unwrap() },
            workers,
            workers,
        );
        let mut txes = Vec::new();
        for _ in 0..10 {
            let (tx, rx) = oneshot::channel();
            sender.send(rx).await.unwrap();
            txes.push(tx);
        }
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
        sender.shutdown();
        for (i, tx) in txes.into_iter().enumerate() {
            let _ = tx.send(i);
        }
        let mut values = receiver.collect::<Vec<_>>().await;
        values.sort_unstable();
        assert_eq!(values, (0..5).collect::<Vec<_>>());
    }
}
