#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use rust_os::{memory::get_page_table_at_physical_addr, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory::get_active_level_4_table;
    use x86_64::VirtAddr;

    println!("Hello world{}", "!");

    rust_os::init();

    // let (level_4_page_table, _cr3_flags) = Cr3::read();

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

    let level_4_table = unsafe { get_active_level_4_table(physical_offset) };

    for (i, entry) in level_4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 entry {}: {:?}", i, entry);

            let physical_addr = entry.frame().unwrap().start_address();

            let level_3_table =
                unsafe { get_page_table_at_physical_addr(&physical_addr, &physical_offset) };

            for (i, entry) in level_3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("    L3 entry {}: {:?}", i, entry);

                    // let physical_addr = entry.frame().unwrap().start_address();

                    // let level_2_table = unsafe {
                    //     get_page_table_at_physical_addr(&physical_addr, &physical_offset)
                    // };

                    // for (i, entry) in level_2_table.iter().enumerate() {
                    //     if !entry.is_unused() {
                    //         println!("        L2 entry {}: {:?}", i, entry);
                    //     }
                    // }
                }
            }
        }
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
