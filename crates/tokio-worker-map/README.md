This crate defines the following function for mapping values through
asynchronous worker tasks:

```rust
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
```

`worker_map()` spawns `workers` concurrent tasks that loop over the values sent
to the sender, apply `func` to them, and send the results to the receiver.  The
sender is clonable, but the receiver is not.  If any application of `func`
panics, the panic is reraised by the receiver.  Once the sender and all of its
clones have been dropped, and once the results of all function applications
have been received, the receiver stream will close.  If the receiver is
dropped, the worker tasks are aborted.
