use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {

    pub fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .flat_map(|r| {
                let start_frame = PhysFrame::containing_address(PhysAddr::new(r.range.start_addr()));
                let end_frame = PhysFrame::containing_address(PhysAddr::new(r.range.end_addr() - 1));
                PhysFrame::range_inclusive(start_frame, end_frame)
            })
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next)?;
        self.next += 1;
        Some(frame)
    }

    pub fn deallocate_frame(&mut self, _frame: PhysFrame) {

    }
}

unsafe impl x86_64::structures::paging::FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frame()
    }
}
