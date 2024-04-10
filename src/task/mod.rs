use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

extern crate alloc;

use alloc::boxed::Box;

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

// A type wrapper around a pinned, heap-allocated, dynamically dispatched
// future, whose output type is an Empty. A task is thus executed for its
// side-effects, and not to produce a useful return value.
pub struct Task {
    id: TaskId,

    // The Box holds a dynamically dispatched trait object, so that
    // different tasks can use different types of Futures.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // The returned Task instance may have an arbitrary lifetime, so the
    // lifetime of the surrounding Box (wrapper) must outlive it.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    // Polls the Future, providing the given Context.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        // Atomically increment NEXT_ID, returning its previous value.

        // The ordering dictates whether or not the Rust compiler may re-order
        // the `fetch_add()` instruction in the instruction stream.
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
