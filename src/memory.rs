use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

//  Consists of a list of MemoryRegion structs, which contain the start address,
//  the length, and the type (unused, reserved, etc.) of each memory region.
use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap, // Memory map provided by our bootloader.
    next: usize,                    // The number of the next frame that the
                                    // allocator should return.
}

impl BootInfoFrameAllocator {
    // Unsafe because the caller must guarantee that the given memory map is
    // valid. All frames marked as `USABLE` really must be unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable (page-aligned) regions from the memory map.
        let regions = self.memory_map.iter();

        let usable_regions =
            regions.filter(|region| region.region_type == MemoryRegionType::Usable);

        // Map each region to its address range.
        let address_ranges =
            usable_regions.map(|region| region.range.start_addr()..region.range.end_addr());

        // Flatten to a list of start addresses.
        let frame_addresses = address_ranges.flat_map(|range| range.step_by(4096));

        // Convert the start addresses to PhysFrame types.
        frame_addresses.map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // @NOTE Not terribly efficient, as we're recreating `usable_frames`
        // iterator on each allocation.
        let frame = self.usable_frames().nth(self.next);

        self.next += 1;

        frame
    }
}

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
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));

    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // @TODO Fix as this is unsafe (may map multiple writable pages to the same frame!)
        mapper.map_to(page, frame, flags, frame_allocator)
    };

    // Flushes the newly mapped page from the translation lookaside buffer.
    map_to_result.expect("map_to failed").flush();
}
