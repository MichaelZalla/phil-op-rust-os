use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

pub unsafe fn get_active_level_4_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_page_table_frame, _cr3_flags) = Cr3::read();

    let physical_addr = level_4_page_table_frame.start_address();

    return get_page_table_at_physical_addr(&physical_addr, &physical_offset);
}

pub unsafe fn get_page_table_at_physical_addr(
    addr: &PhysAddr,
    physical_offset: &VirtAddr,
) -> &'static mut PageTable {
    let virtual_addr = *physical_offset + addr.as_u64();

    let page_table: *mut PageTable = virtual_addr.as_mut_ptr();

    &mut *page_table // unsafe
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_offset)
}

unsafe fn translate_addr_inner(addr: VirtAddr, physical_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_4_page_table_frame, _) = Cr3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    // Points to page table frames, until the last iteration—at which point,
    // `current_frame` will point to the mapped physical frame.
    let mut current_frame = level_4_page_table_frame;

    for &index in &table_indices {
        // Converts the frame into a page table reference.
        let virtual_addr = physical_offset + current_frame.start_address().as_u64();

        let table_ptr: *const PageTable = virtual_addr.as_ptr();

        let table = unsafe { &*table_ptr };
        let entry = &table[index];

        // Reads the page table entry and updates `current_frame`.
        current_frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages are not supported."),
        };
    }

    Some(current_frame.start_address() + u64::from(addr.page_offset()))
}
