This crate defines a [tokio][]-based *task group* or *nursery* for spawning
asynchronous tasks and retrieving their return values.  The API is based on
[`async_nursery`][], which would have been perfect for my needs at the time,
except that it doesn't support creating a nursery inside a Tokio runtime.

Usage
=====

Call `Nursery::new()` to receive a `(Nursery<T>, NurseryStream<T>)` pair, where
`T` is the output type of the futures that you'll be spawning in the nursery.
Call `nursery.spawn(future)` to spawn a future.  The nursery is clonable &
sendable, and so it can be used to spawn tasks from within other tasks.  You
can even create a nursery inside a task of another nursery.

The `NurseryStream` is a [`Stream`][] of the values returned by the tasks as
they complete; if a task panics, the panic is propagated.  Once the `Nursery`
object and all of its clones have been dropped, and once all spawned futures
have completed, the stream will close.  If the `NurseryStream` is dropped, all
tasks in the nursery are aborted.

[tokio]: https://tokio.rs
[`async_nursery`]: https://crates.io/crates/async_nursery
[`Stream`]: https://docs.rs/futures-util/latest/futures_util/stream/trait.Stream.html
