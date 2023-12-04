use self::inner::Receiver as _;
use futures_util::{
    future::{maybe_done, MaybeDone},
    stream::Stream,
};
use pin_project_lite::pin_project;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};

/// `received_stream()` takes a buffer size and an async procedure that takes a
/// [`tokio::sync::mpsc::Sender`], and it returns a stream that runs the
/// procedure to completion while yielding the values passed to the sender.
///
/// If the stream is dropped before completion, the async procedure (which may
/// or may not have completed by that point) is dropped as well.
pub fn received_stream<F, Fut, T>(buffer: usize, f: F) -> ReceivedStream<Fut, T, Receiver<T>>
where
    F: FnOnce(Sender<T>) -> Fut,
    Fut: Future<Output = ()>,
{
    let (sender, receiver) = channel(buffer);
    let future = f(sender);
    ReceivedStream::new(future, receiver)
}

/// `unbounded_received_stream()` takes an async procedure that takes a
/// [`tokio::sync::mpsc::UnboundedSender`], and it returns a stream that runs
/// the procedure to completion while yielding the values passed to the sender.
///
/// If the stream is dropped before completion, the async procedure (which may
/// or may not have completed by that point) is dropped as well.
pub fn unbounded_received_stream<F, Fut, T>(f: F) -> ReceivedStream<Fut, T, UnboundedReceiver<T>>
where
    F: FnOnce(UnboundedSender<T>) -> Fut,
    Fut: Future<Output = ()>,
{
    let (sender, receiver) = unbounded_channel();
    let future = f(sender);
    ReceivedStream::new(future, receiver)
}

pin_project! {
    pub struct ReceivedStream<Fut, T, Recv> where Fut: Future {
        #[pin]
        future: MaybeDone<Fut>,
        receiver: inner::MaybeAllReceived<Recv>,
        _item: PhantomData<T>,
    }
}

impl<Fut: Future, T, Recv> ReceivedStream<Fut, T, Recv> {
    fn new(future: Fut, receiver: Recv) -> Self {
        ReceivedStream {
            future: maybe_done(future),
            receiver: inner::MaybeAllReceived::InProgress(receiver),
            _item: PhantomData,
        }
    }
}

impl<Fut, T, Recv> Stream for ReceivedStream<Fut, T, Recv>
where
    Fut: Future<Output = ()>,
    Recv: inner::Receiver<Item = T>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let this = self.project();
        let fut_poll = this.future.poll(cx).map(|()| None);
        let recv_poll = this.receiver.poll_next_recv(cx);
        if recv_poll.is_pending() {
            fut_poll
        } else {
            recv_poll
        }
    }
}

mod inner {
    use std::task::{Context, Poll};
    use tokio::sync::mpsc;

    pub(super) trait Receiver {
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

    pub(super) enum MaybeAllReceived<Recv> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
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
