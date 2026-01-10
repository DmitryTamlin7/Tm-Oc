use x86_64::{structures::paging::{OffsetPageTable, PageTable, PhysFrame, Mapper, Size4KiB, Page, PageTableFlags, FrameAllocator}, VirtAddr, PhysAddr,};

pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // небезопасное разыменование
}

pub fn identity_map_range(
    start_addr: u64,
    end_addr: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), x86_64::structures::paging::mapper::MapToError<Size4KiB>> {
    let start_page = Page::containing_address(VirtAddr::new(start_addr));
    let end_page = Page::containing_address(VirtAddr::new(end_addr - 1));

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = PhysFrame::containing_address(PhysAddr::new(page.start_address().as_u64()));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }
    Ok(())
}

