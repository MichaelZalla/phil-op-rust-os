use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};

use conquer_once::spin::OnceCell;

use crossbeam_queue::ArrayQueue;

use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{print, println};

static WAKER: AtomicWaker = AtomicWaker::new();

// We avoid using the `lazy_static!` macro to initialize this static, as it will
// be first accessed inside our interrupt handler, and we want to avoid
// performing a heap allocating from within this handler—because doing so would
// require a lock on our heap allocator, which may cause deadlocks.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue is full! Dropping keyboard input.");
        } else {
            // Calls `wake()` on the last Waker passed to
            // `AtomicWaker::register()`; this notifies our executor. If no
            // Waker has been registered to our `WAKER`, this call is a no-op.

            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized!");
    }
}

pub struct ScancodeStream {
    // Prevents external modules from trying to make a ScancodeStream without
    // using the ScancodeStream::new() path.
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new() should only be called once!");

        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue uninitialized!");

        if let Ok(scancode) = queue.pop() {
            // Fast path
            return Poll::Ready(Some(scancode));
        }

        // First call to `queue.pop()` didn't succeed.

        // Our queue _might_ be empty—or the keyboard interrupt handler may have
        // enqueued a new item just after the pop() operation above.

        // Registers our task context's Waker with the static `AtomicWaker`
        // instance. This ensures that

        WAKER.register(&cx.waker());

        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();

                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypresses_task() {
    let mut scancodes = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Asynchronously wait for the result of each Future returned by
    // ScancodeStream::next(). Since ScancodeStream::next() never returns None,
    // this means that our print_keypresses() task runs indefinitely.

    while let Some(scancode) = scancodes.next().await {
        // Processes an 8-bit scancode into a keyboard event.

        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // Produces a decoded key from the key event.

            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
