use futures_util::{
    future::{MaybeDone, maybe_done},
    stream::Stream,
};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::{sync::mpsc, task::JoinHandle};

/// `received_stream()` takes a buffer size and an async function that takes a
/// [`tokio::sync::mpsc::Sender`], and it spawns a task for the function and
/// returns a stream of the values passed to the sender.
///
/// If the stream is dropped before completion, the async function is cancelled.
pub fn received_stream<F, Fut, T>(buffer: usize, f: F) -> ReceivedStream<mpsc::Receiver<T>>
where
    F: FnOnce(mpsc::Sender<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(buffer);
    let handle = tokio::spawn(f(sender));
    ReceivedStream::new(handle, receiver)
}

/// `unbounded_received_stream()` takes an async function that takes a
/// [`tokio::sync::mpsc::UnboundedSender`], and it spawns a task for the
/// function and returns a stream of the values passed to the sender.
///
/// If the stream is dropped before completion, the async function is cancelled.
pub fn unbounded_received_stream<F, Fut, T>(f: F) -> ReceivedStream<mpsc::UnboundedReceiver<T>>
where
    F: FnOnce(mpsc::UnboundedSender<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = mpsc::unbounded_channel();
    let handle = tokio::spawn(f(sender));
    ReceivedStream::new(handle, receiver)
}

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct ReceivedStream< Recv> {
        #[pin]
        handle: MaybeDone<FragileHandle<()>>,
        receiver: MaybeAllReceived<Recv>,
    }
}

impl<Recv> ReceivedStream<Recv> {
    fn new(handle: JoinHandle<()>, receiver: Recv) -> Self {
        ReceivedStream {
            handle: maybe_done(FragileHandle::new(handle)),
            receiver: MaybeAllReceived::InProgress(receiver),
        }
    }
}

impl<Recv> Stream for ReceivedStream<Recv>
where
    Recv: Receiver,
{
    type Item = <Recv as Receiver>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let fut_poll = this.handle.poll(cx).map(|()| None);
        let recv_poll = this.receiver.poll_next_recv(cx);
        if recv_poll.is_pending() {
            fut_poll
        } else {
            recv_poll
        }
    }
}

pub trait Receiver {
    type Item;

    fn poll_next_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

impl<T> Receiver for mpsc::Receiver<T> {
    type Item = T;

    fn poll_next_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.poll_recv(cx)
    }
}

impl<T> Receiver for mpsc::UnboundedReceiver<T> {
    type Item = T;

    fn poll_next_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.poll_recv(cx)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MaybeAllReceived<Recv> {
    InProgress(Recv),
    Done,
}

impl<Recv: Receiver> Receiver for MaybeAllReceived<Recv> {
    type Item = <Recv as Receiver>::Item;

    fn poll_next_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self {
            MaybeAllReceived::InProgress(recv) => {
                let p = recv.poll_next_recv(cx);
                if matches!(p, Poll::Ready(None)) {
                    *self = MaybeAllReceived::Done;
                }
                p
            }
            MaybeAllReceived::Done => Poll::Ready(None),
        }
    }
}

pin_project! {
    /// A wrapper around `tokio::task::JoinHandle` that aborts the task on drop.
    #[derive(Debug)]
    struct FragileHandle<T> {
        #[pin]
        inner: JoinHandle<T>
    }

    impl<T> PinnedDrop for FragileHandle<T> {
        fn drop(this: Pin<&mut Self>) {
            this.project().inner.abort();
        }
    }
}

impl<T> FragileHandle<T> {
    fn new(inner: JoinHandle<T>) -> Self {
        FragileHandle { inner }
    }
}

impl<T> Future for FragileHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;
    use std::io::Cursor;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::io::AsyncBufReadExt;

    #[tokio::test]
    async fn test_received_stream() {
        let done = Arc::new(AtomicBool::new(false));
        let inner_done = done.clone();
        let stream = received_stream(5, |sender| async move {
            let cursor = Cursor::new("0 1 2 3 4 5 6 7\n8 9 10\n11 12 13 14\n");
            let mut lines = cursor.lines();
            while let Some(ln) = lines.next_line().await.unwrap() {
                for n in ln
                    .split_ascii_whitespace()
                    .map(|s| s.parse::<usize>().unwrap())
                {
                    if sender.send(n).await.is_err() {
                        return;
                    }
                }
            }
            inner_done.store(true, Ordering::Relaxed);
        })
        .enumerate();
        tokio::pin!(stream);
        while let Some((i, n)) = stream.next().await {
            assert_eq!(i, n);
        }
        assert!(done.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_received_stream_drop() {
        let done = Arc::new(AtomicBool::new(false));
        let inner_done = done.clone();
        let stream = received_stream(5, |sender| async move {
            let cursor = Cursor::new("0 1 2 3 4 5 6 7\n8 9 10\n11 12 13 14\n");
            let mut lines = cursor.lines();
            while let Some(ln) = lines.next_line().await.unwrap() {
                for n in ln
                    .split_ascii_whitespace()
                    .map(|s| s.parse::<usize>().unwrap())
                {
                    if sender.send(n).await.is_err() {
                        return;
                    }
                }
            }
            inner_done.store(true, Ordering::Relaxed);
        });
        tokio::pin!(stream);
        assert_eq!(stream.next().await, Some(0));
        #[allow(clippy::drop_non_drop)]
        drop(stream);
        assert!(!done.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_unbounded_received_stream() {
        let done = Arc::new(AtomicBool::new(false));
        let inner_done = done.clone();
        let stream = unbounded_received_stream(|sender| async move {
            let cursor = Cursor::new("0 1 2 3 4 5 6 7\n8 9 10\n11 12 13 14\n");
            let mut lines = cursor.lines();
            while let Some(ln) = lines.next_line().await.unwrap() {
                for n in ln
                    .split_ascii_whitespace()
                    .map(|s| s.parse::<usize>().unwrap())
                {
                    if sender.send(n).is_err() {
                        return;
                    }
                }
            }
            inner_done.store(true, Ordering::Relaxed);
        })
        .enumerate();
        tokio::pin!(stream);
        while let Some((i, n)) = stream.next().await {
            assert_eq!(i, n);
        }
        assert!(done.load(Ordering::Relaxed));
    }
}
