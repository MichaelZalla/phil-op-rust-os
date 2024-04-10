extern crate alloc;

use core::task::{Context, Poll, Waker};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};

use x86_64::instructions::interrupts::{self, enable_and_hlt};

use crossbeam_queue::ArrayQueue;

use super::{Task, TaskId};

pub struct Executor {
    // Fast access to tasks via ID lookup.
    tasks: BTreeMap<TaskId, Task>,

    // Reference-counted, so it can be shared between executor and wakers; we
    // use a fixed-size queue so that interrupt handlers can push to it without
    // making heap allocations (potential for deadlock).
    task_queue: Arc<ArrayQueue<TaskId>>,

    // Fast access to wakers via ID lookup; lets us re-use any existing waker
    // instance for subsequent wake-ups of a given task; also, holding one
    // reference to each waker from inside our executor prevents a waker from
    // being de-allocated from inside an interrupt handler (i.e., deadlock).
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::<TaskId, Task>::new(),
            task_queue: Arc::new(ArrayQueue::<TaskId>::new(100)),
            waker_cache: BTreeMap::<TaskId, Waker>::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task_id, task).is_some() {
            panic!("Task with ID {} already exists.", task_id.0);
        };

        self.task_queue.push(task_id).expect("Task queue full.");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();

            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();

        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }

    fn run_ready_tasks(&mut self) {
        // For each task presently in the task queue...
        while let Ok(task_id) = self.task_queue.pop() {
            // Verify that the task still exists.
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // Task no longer exists!
            };

            // Retrieves a pre-allocated task waker, or creates a new one and
            // caches it. Note that our `task_queue` is wrapped in an `Arc`, so
            // calls to `clone()` simply increment the reference count.
            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone()));

            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);

                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    // Uses the `Waker::from()`` implementation to convert `Self`` to a `Waker`.
    // The inner method creates a RawWakerTable and a RawWaker instance.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id)
            .expect("Task queue is full!");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}
