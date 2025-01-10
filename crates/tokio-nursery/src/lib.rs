use futures_util::{FutureExt, Stream};
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

type UnwindResult<T> = Result<T, Box<dyn std::any::Any + Send>>;

#[derive(Debug)]
pub struct Nursery<T> {
    sender: UnboundedSender<UnwindResult<T>>,
}

impl<T: Send + 'static> Nursery<T> {
    pub fn new() -> (Nursery<T>, NurseryStream<T>) {
        let (sender, receiver) = unbounded_channel();
        (Nursery { sender }, NurseryStream { receiver })
    }

    pub fn spawn<Fut>(&self, fut: Fut)
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let task = std::panic::AssertUnwindSafe(fut).catch_unwind();
            let _ = sender.send(task.await);
        });
    }
}

// Clone can't be derived, as that would erroneously add `T: Clone` bounds to
// the impl.
impl<T> Clone for Nursery<T> {
    fn clone(&self) -> Nursery<T> {
        Nursery {
            sender: self.sender.clone(),
        }
    }
}

#[derive(Debug)]
pub struct NurseryStream<T> {
    receiver: UnboundedReceiver<UnwindResult<T>>,
}

impl<T: 'static> Stream for NurseryStream<T> {
    type Item = T;

    /// Poll for one of the tasks in the group to complete and return its
    /// return value.
    ///
    /// # Panics
    ///
    /// If a task panics, this method resumes unwinding the panic.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match ready!(self.receiver.poll_recv(cx)) {
            Some(Ok(r)) => Some(r).into(),
            Some(Err(e)) => std::panic::resume_unwind(e),
            None => None.into(),
        }
    }
}
