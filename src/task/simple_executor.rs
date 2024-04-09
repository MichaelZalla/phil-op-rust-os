use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

extern crate alloc;

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>, // FIFO work queue
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            // Initializes an empty task queue.
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        // Enqueues this task.
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            // Creates a Waker instance from a RawWaker.
            let waker = dummy_waker();

            // Wraps the Waker in a Context object.
            let mut context = Context::from_waker(&waker);

            // Polls this task, providing the Context.
            match task.poll(&mut context) {
                Poll::Ready(()) => {}                             // Task is finished!
                Poll::Pending => self.task_queue.push_back(task), // Task is still running...
            }
        }
    }
}

// Dummy Waker implementation using no-ops for `wake()` and `drop()`.
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}

    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // Signature:
    //   RawWakerVTable::new(clone, wake, wake_by_ref, drop) -> RawWakerVTable
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    RawWaker::new(0 as *const (), vtable)
}
