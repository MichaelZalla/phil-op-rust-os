#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use rust_os::{
    allocator, println,
    task::{executor::Executor, keyboard::print_keypresses_task, Task},
};
use x86_64::structures::paging::Page;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::VirtAddr;

    println!("Hello world{}", "!");

    rust_os::init();

    let physical_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { memory::init(physical_offset) };

    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));

    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();

    unsafe {
        // Writes the string "New!" to the VGA text buffer, via the mapping.
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed.");

    // Initializes our task executor.
    let mut executor = Executor::new();

    // Moves a Future to the heap and pins it.
    let example_task_pinned = Task::new(example_task());

    // Enqueues an example task in the executor's work queue.
    executor.spawn(example_task_pinned);

    // Moves a Future to the heap and pins it.
    let print_keypresses_task_pinned = Task::new(print_keypresses_task());

    // Enqueues a print-keypresses task.
    executor.spawn(print_keypresses_task_pinned);

    // Polls tasks until all tasks are complete.
    executor.run();

    #[cfg(test)]
    test_main();

    // println!("It didn't crash!");

    // rust_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;

    println!("async number is {}", number);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    rust_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}
