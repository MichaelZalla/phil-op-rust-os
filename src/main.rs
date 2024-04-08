#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use rust_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    rust_os::init();

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _cr3_flags) = Cr3::read();

    println!(
        "Level 4 page table exists at: {:?}.",
        level_4_page_table.start_address()
    );

    // // Triggers a breakpoint interrupt.
    // x86_64::instructions::interrupts::int3();

    // Triggers a page-fault exception.
    unsafe {
        // *(0xdeadbeef as *mut u8) = 42;

        let code_page = 0x204944 as *mut u8;

        let _x = *code_page;
        println!("Read worked!");

        *code_page = 42;
        println!("Write worked!");
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
