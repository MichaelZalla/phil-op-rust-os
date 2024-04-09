use conquer_once::spin::OnceCell;

use crossbeam_queue::ArrayQueue;

use crate::println;

// We avoid using the `lazy_static!` macro to initialize this static, as it will
// be first accessed inside our interrupt handler, and we want to avoid
// performing a heap allocating from within this handler—because doing so would
// require a lock on our heap allocator, which may cause deadlocks.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue is full! Dropping keyboard input.");
        }
    } else {
        println!("WARNING: scancode queue uninitialized!");
    }
}
