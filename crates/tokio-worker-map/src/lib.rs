use futures_util::{FutureExt, Stream};
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

type UnwindResult<T> = Result<T, Box<dyn std::any::Any + Send>>;

pub fn worker_map<F, Fut, T, U>(
    func: F,
    workers: NonZeroUsize,
    buffer_size: NonZeroUsize,
) -> (Sender<T>, Receiver<U>)
where
    F: Fn(T) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = U> + Send,
    T: Send + 'static,
    U: Send + 'static,
{
    let (input_sender, input_receiver) = async_channel::bounded(buffer_size.get());
    let (output_sender, output_receiver) = tokio::sync::mpsc::channel(buffer_size.get());
    for _ in 0..workers.get() {
        tokio::spawn({
            let func = func.clone();
            let input = input_receiver.clone();
            let output = output_sender.clone();
            async move {
                while let Ok(work) = input.recv().await {
                    let r = std::panic::AssertUnwindSafe(func(work))
                        .catch_unwind()
                        .await;
                    if output.send(r).await.is_err() {
                        break;
                    }
                }
            }
        });
    }
    (Sender(input_sender), Receiver(output_receiver))
}

#[derive(Debug)]
pub struct Sender<T>(async_channel::Sender<T>);

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> async_channel::Send<'_, T> {
        self.0.send(msg)
    }
}

// Clone can't be derived, as that would erroneously add `T: Clone` bounds to
// the impl.
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        Sender(self.0.clone())
    }
}

#[derive(Debug)]
pub struct Receiver<T>(tokio::sync::mpsc::Receiver<UnwindResult<T>>);

impl<T: Send> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        match self.0.recv().await? {
            Ok(r) => Some(r),
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}

impl<T: 'static> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match ready!(self.0.poll_recv(cx)) {
            Some(Ok(r)) => Some(r).into(),
            Some(Err(e)) => std::panic::resume_unwind(e),
            None => None.into(),
        }
    }
}
