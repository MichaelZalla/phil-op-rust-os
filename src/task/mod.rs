use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

extern crate alloc;

use alloc::boxed::Box;

pub mod keyboard;
pub mod simple_executor;

// A type wrapper around a pinned, heap-allocated, dynamically dispatched
// future, whose output type is an Empty. A task is thus executed for its
// side-effects, and not to produce a useful return value.
pub struct Task {
    // The Box holds a dynamically dispatched trait object, so that
    // different tasks can use different types of Futures.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // The returned Task instance may have an arbitrary lifetime, so the
    // lifetime of the surrounding Box (wrapper) must outlive it.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    // Polls the Future, providing the given Context.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
