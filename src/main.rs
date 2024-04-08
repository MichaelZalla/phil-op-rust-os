#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};

use rust_os::{
    allocator::{self, HEAP_START},
    println,
};
use x86_64::structures::paging::Page;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::{
        // structures::paging::Translate,
        VirtAddr,
    };

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

    let _heap_start = unsafe { *(HEAP_START as *const u32) };

    // Allocate a (boxed) i32 on our heap.

    let heap_value = Box::new(42);
    println!("Box<i32> at {:p}", heap_value);

    // Allocate and grow a vec on our heap.

    let mut vec = Vec::new();

    for i in 0..500 {
        vec.push(i);
    }

    println!("Vec<i32> at {:p}", vec.as_slice());

    // Allocate a reference-counted vec.

    let reference_counted = Rc::new(vec![1, 2, 3]);

    println!("Rc<Vec<i32>> at {:p}", reference_counted);

    // Cloned our reference to the (reference-counted) vec.

    let cloned_reference = reference_counted.clone();

    println!("Rc<Vec<i32>> (cloned) at {:p}", cloned_reference);

    println!(
        "Original reference count: {}",
        Rc::strong_count(&cloned_reference)
    );

    // Drop the original reference, and inspect the reference count.

    core::mem::drop(reference_counted);

    println!(
        "New reference count: {}",
        Rc::strong_count(&cloned_reference)
    );

    #[cfg(test)]
    test_main();

    println!("It didn't crash!");

    rust_os::hlt_loop();
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
