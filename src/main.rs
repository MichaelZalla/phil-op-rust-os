#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_buffer::print_something();

    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    panic!("Some panic message");
}
