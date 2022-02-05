use rv64::csr::satp::{Satp, SatpMode};
use rv64::asm::sfence_vma;

use crate::vm::page_table::{PageTable, PageTableLevel};
use crate::vm::addr::{VirtAddr, PhysAddr};
use crate::vm::page_flag::PteFlag;
use crate::riscv::{PAGESIZE, MAXVA};
use crate::memorylayout::{UART0, PLIC_BASE, TRAMPOLINE, KERNELBASE, PHYSTOP};
use crate::kalloc::kalloc;

static mut KERNELPAGE: Option<&mut PageTable> = None;

pub fn init_kvm() {
    extern "C" {
        static _trampoline: usize;
        static _etext: usize;
    }
    let ptrampoline: u64 = unsafe { &_trampoline as *const usize as u64 };
    let petext: u64 = unsafe { &_etext as *const usize as u64 };
    unsafe {
        KERNELPAGE = Some(&mut *(kalloc() as *mut PageTable));
    }

    // map UART registers
    kvmmap(VirtAddr::new(UART0), PhysAddr::new(UART0), PAGESIZE,
           PteFlag::PTE_READ | PteFlag::PTE_WRITE);

    // map PLIC registers
    kvmmap(VirtAddr::new(PLIC_BASE), PhysAddr::new(PLIC_BASE), 0x400000, PteFlag::PTE_READ | PteFlag::PTE_WRITE);

    // map kernel text read and executable
    kvmmap(VirtAddr::new(KERNELBASE), PhysAddr::new(KERNELBASE), petext - KERNELBASE,
           PteFlag::PTE_READ | PteFlag::PTE_EXEC);

    // map kernel data and physical ram
    kvmmap(VirtAddr::new(petext), PhysAddr::new(petext), PHYSTOP as u64 - petext,
           PteFlag::PTE_READ | PteFlag::PTE_WRITE);

    // map the trampoline to the highest virtual address in the kernel
    // for trap enter/exit
    kvmmap(VirtAddr::new(TRAMPOLINE), PhysAddr::new(ptrampoline), PAGESIZE,
           PteFlag::PTE_READ | PteFlag::PTE_EXEC);
}

pub fn init_page() {
    let mut satp = Satp::from_bits(0);
    let ptr = unsafe { get_root_page() };
    satp.set_mode(SatpMode::ModeSv39);
    satp.set_addr(ptr as *const _ as u64);
    satp.write();
    sfence_vma();
}

pub unsafe fn get_root_page() -> &'static mut PageTable {
    KERNELPAGE.as_mut().unwrap()
}

/// Add a mapping to the kernel page table.
/// only used when booting before enable paging.
fn kvmmap(va: VirtAddr, pa: PhysAddr, size: u64, perm: PteFlag) {
    match map_pages(va, pa, size, perm) {
        Ok(_) => {},
        Err(e) => panic!("mappages error: {}", e),
    }
}

/// Create PTEs for virtual addresses starting at va that refer to
/// physical addresses starting at pa. va and size might not
/// be page-aligned.
/// Return Errs if it cannot allocate the needed page-table.
fn map_pages(va: VirtAddr, mut pa: PhysAddr, size: u64, perm: PteFlag) -> Result<(), &'static str> {
    let page_table = unsafe { get_root_page() };
    let va_start = va.align_down();
    let va_end = VirtAddr::new_truncate(va.as_u64() + size - 1).align_down();
    let mut page_addr = va_start;

    loop {
        map_page(page_table, page_addr, pa, perm, PageTableLevel::Two)?;
        if page_addr == va_end {
            break;
        }
        page_addr += PAGESIZE;
        pa += PAGESIZE;
    }

    Ok(())
}

fn map_page(page_table: &mut PageTable, va: VirtAddr, pa: PhysAddr, perm: PteFlag, level: PageTableLevel) -> Result<(), &'static str> {
    if va >= VirtAddr::new(MAXVA) {
        return Err("map_page: virtual address over MAX address")
    }
    let index = va.get_index(level);
    let pte = &mut page_table[index];
    match level.next_level() {
        None => {
            // Recursive end, write pte or error because of remap
            if pte.is_unused() {
                pte.set_addr(pa.as_pte(), perm | PteFlag::PTE_VALID);
                Ok(())
            } else {
                Err("map_page: remap")
            }
        },
        Some(next_level) => {
            // Allocate space for page table and call map_page with next level
            if pte.is_unused() {
                let ptr = kalloc();
                if ptr == 0 as *mut u8 {
                    return Err("kalloc failed in map_page");
                }
                let addr = PhysAddr::new(ptr as *const _ as u64);
                pte.set_addr(addr.as_pte(), PteFlag::PTE_VALID);
            }
            let next_table = unsafe { &mut *(pte.addr() as *mut PageTable) };
            map_page(next_table, va, pa, perm, next_level)
        }
    }
}
