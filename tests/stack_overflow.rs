#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use lazy_static::lazy_static;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use rust_os::{exit_qemu, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rust_os::gdt::init();
    // rust_os::interrupts::init_idt();
    init_test_idt();

    // Triggers a triple fault (QEMU system reboot)
    stack_overflow();

    panic!("Execution continued after stack overflow!");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    // For each recursion, the return address is pushed.
    stack_overflow();

    // Prevents tail-recursion optimizations.
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_print!("[ok]");

    exit_qemu(rust_os::QemuExitCode::Success);

    loop {}
}
