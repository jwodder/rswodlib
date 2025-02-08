This crate defines a [tokio][]-based *task group* or *nursery* for spawning
futures that run on a fixed number of worker tasks.

Usage
=====

Call `WorkerNursery::new(workers)` to receive a `(WorkerNursery<T>,
WorkerNurseryStream<T>)` pair, where `T` is the output type of the futures that
you'll be spawning in the nursery and `workers` is the number of worker tasks
that the nursery will use for executing futures.  Call `nursery.spawn(future)`
to spawn a future on one of the workers.  The nursery is clonable & sendable,
and so it can be used to spawn tasks from within other tasks.  You can even
create a nursery inside a future of another nursery.

The `WorkerNurseryStream` is a [`Stream`][] of the values returned by the
futures as they complete; if a future panics, the panic is propagated.  Once
the `WorkerNursery` object and all of its clones have been dropped, and once
all spawned futures have completed, the stream will close.  If the
`WorkerNurseryStream` is dropped, all tasks in the nursery are aborted.

[tokio]: https://tokio.rs
[`Stream`]: https://docs.rs/futures-util/latest/futures_util/stream/trait.Stream.html
