use x86_64::{
    instructions::segmentation::{CS, Segment},
    instructions::tables::load_tss,
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    structures::tss::TaskStateSegment,
    VirtAddr,
};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_end = VirtAddr::from_ptr(unsafe { &STACK as *const _ })
            + STACK_SIZE as u64;

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT_AND_SELECTORS: (GlobalDescriptorTable, Selectors) = {

        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&*TSS));

        (gdt, Selectors { code_selector, tss_selector })
    };
}

pub fn init() {

    GDT_AND_SELECTORS.0.load();

    unsafe {
        CS::set_reg(GDT_AND_SELECTORS.1.code_selector);
        load_tss(GDT_AND_SELECTORS.1.tss_selector);
    }
}