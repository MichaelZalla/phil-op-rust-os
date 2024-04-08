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

    // // Triggers a breakpoint interrupt.
    // x86_64::instructions::interrupts::int3();

    // // Triggers a page fault exception.
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

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
