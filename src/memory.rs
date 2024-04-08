use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

pub unsafe fn init(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_page_table = get_active_level_4_table(physical_offset);

    OffsetPageTable::new(level_4_page_table, physical_offset)
}

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
