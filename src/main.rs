#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use rust_os::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    println!("Hello world{}", "!");

    rust_os::init();

    // println!(
    //     "Level 4 page table exists at: {:?}.",
    //     level_4_page_table.start_address()
    // );

    // // Triggers a breakpoint interrupt.
    // x86_64::instructions::interrupts::int3();

    // Triggers a page-fault exception.
    // unsafe {
    //     // *(0xdeadbeef as *mut u8) = 42;

    //     let code_page = 0x204944 as *mut u8;

    //     let _x = *code_page;
    //     println!("Read worked!");

    //     *code_page = 42;
    //     println!("Write worked!");
    // }

    let physical_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mapper = unsafe { memory::init(physical_offset) };

    let addresses = [
        // Memory-mapped VGA text buffer
        0xb8000,
        // Some code page
        0x201008,
        // Some stack page
        0x0100_0020_1a10,
        // Virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &addr in &addresses {
        let virtual_addr = VirtAddr::new(addr);

        let physical_addr = mapper.translate(virtual_addr);

        println!("{:?} -> {:?}", virtual_addr, physical_addr);
    }

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
