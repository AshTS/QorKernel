pub mod addresses;
pub mod entry;
pub mod table;

/// Set a page table as the currently used page table
pub fn set_page_table(table: &'static mut table::PageTable) {
    let addr = table as *mut table::PageTable as usize;
    unsafe { riscv::register::satp::set(riscv::register::satp::Mode::Sv39, 0, addr >> 12) }
}
