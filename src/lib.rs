#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());

        self();

        serial_println!("[ok]");
    }
}

// `tests` is a slice of trait object references of the Testable trait
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests:", tests.len());

    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

// Entrypoint for `cargo test --lib`
#[cfg(test)]
entry_point!(kernel_main);

#[cfg(test)]
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();

    test_main();

    hlt_loop();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");

    serial_println!("Error: {}\n", info);

    exit_qemu(QemuExitCode::Failed);

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // (0x10 << 1) | 1 = (16 * 2) + 1 = 33
    Failed = 0x11,  // (0x11 << 1) | 1 = (17 * 2) + 1 = 35
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);

        port.write(exit_code as u32);
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }
    x86_64::instructions::interrupts::enable();
}
