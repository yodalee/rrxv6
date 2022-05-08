use rv64::csr::satp::{Satp, SatpMode};
use rv64::asm::sfence_vma;

use crate::vm::page_table::{PageTable, PageTableLevel};
use crate::vm::addr::{VirtAddr, PhysAddr};
use crate::vm::page_flag::PteFlag;
use crate::riscv::{PAGESIZE, MAXVA};
use crate::memorylayout::{UART0, PLIC_BASE, TRAMPOLINE, TRAPFRAME, KERNELBASE, PHYSTOP, kstack};
use crate::kalloc::{kalloc, kfree};
use crate::param::NPROC;
use crate::proc::Proc;

use core::ptr::{NonNull, copy, write_bytes};

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

    // alloc and map stack for kernel process
    for i in 0u64..NPROC as u64 {
        let ptr = kalloc();
        if ptr == 0 as *mut u8 {
            panic!("kalloc failed in alloc proc stack");
        }
        let pa = PhysAddr::new(ptr as *const _ as u64);
        let va = VirtAddr::new(kstack(i));
        kvmmap(va, pa, PAGESIZE, PteFlag::PTE_READ | PteFlag::PTE_WRITE);
    }
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
    let page_table = unsafe { get_root_page() };
    map_pages(page_table, va, pa, size, perm)
        .expect("map_pages_error");
}

/// Create PTEs for virtual addresses starting at va that refer to
/// physical addresses starting at pa. va and size might not
/// be page-aligned.
/// Return Errs if it cannot allocate the needed page-table.
fn map_pages(page_table: &mut PageTable, va: VirtAddr, mut pa: PhysAddr, size: u64, perm: PteFlag) -> Result<(), &'static str> {
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

extern "C" {
    fn trampoline();
}

pub fn init_user_pagetable(proc: &Proc) -> Option<NonNull<PageTable>> {
    // TODO make pagetable full of zero
    if let Some(mut page_table_ptr) = NonNull::new(kalloc() as *mut _) {
        let page_table = unsafe { page_table_ptr.as_mut() };

        // map the trampoline code (for system call return)
        // at the highest user virtual address.
        // only the supervisor uses it, on the way to/from user space, so not PTE_U.
        let trampoline = PhysAddr::new(trampoline as u64);
        if let Err(_e) = map_pages(page_table, VirtAddr::new(TRAMPOLINE), trampoline, PAGESIZE,
            PteFlag::PTE_READ | PteFlag::PTE_EXEC) {
                // TODO uvm free
                // uvmfree(pagetable, 0);
                return None;
        };

        let trapframe = PhysAddr::new(proc.trapframe.as_ptr() as u64);
        if let Err(_e) = map_pages(page_table, VirtAddr::new(TRAPFRAME), trapframe, PAGESIZE,
                PteFlag::PTE_READ | PteFlag::PTE_WRITE) {
                // TODO uvm unmap, free
                // uvmunmap(pagetable, TRAMPOLINE, 1, 0);
                // uvmfree(pagetable, 0);
                return None;
        };
        Some(page_table_ptr)
    } else {
        None
    }
}

pub fn init_uvm(page_table: &mut PageTable, code: &[u8]) {
    let size = code.len();
    let pagesize = PAGESIZE as usize;
    if size > pagesize {
        panic!("init_uvm: more than a page");
    }
    let ptr = kalloc();
    unsafe {
        write_bytes(ptr, 0, pagesize);
        let va = VirtAddr::new(0);
        let pa = PhysAddr::new(ptr as u64);
        let perm = PteFlag::PTE_READ | PteFlag::PTE_WRITE | PteFlag::PTE_EXEC | PteFlag::PTE_USER;
        map_pages(page_table, va, pa, PAGESIZE, perm)
            .expect("init_uvm");
        copy::<u8>(code.as_ptr() as *const u8, ptr, size);
    }
}

pub fn clear_user_pagetable(proc: &mut Proc) {
    unsafe {
        let page_table = proc.pagetable.as_mut();
        unmap_pages(page_table, VirtAddr::new(TRAMPOLINE), 1, false)
            .expect("unmap_pages error");
        unmap_pages(page_table, VirtAddr::new(TRAPFRAME), 1, false)
            .expect("unmap_pages error");
    }
}

/// Remove npages of mappings starting fom va. va must be page-aligned.
/// panic! if mappings is not exist.
/// Optional: free the physical memory.
fn unmap_pages(page_table: &mut PageTable, va: VirtAddr, npages: u64, do_free: bool) -> Result<(), &'static str> {
    if !va.is_align() {
        return Err("unmap_pages: not aligned");
    }

    let mut addr = va;
    while addr < va + npages * PAGESIZE {
        unmap_page(page_table, addr, PageTableLevel::Two, do_free)?;
        addr += PAGESIZE;
    }

    Ok(())
}

fn unmap_page(page_table: &mut PageTable, va: VirtAddr, level: PageTableLevel, do_free: bool) -> Result<(), &'static str> {
    if va >= VirtAddr::new(MAXVA) {
        return Err("unmap_page: virtual address over MAX address")
    }
    let index = va.get_index(level);
    let pte = &mut page_table[index];
    match level.next_level() {
        None => {
            // Recursive end
            if pte.is_unused() {
                Err("unmap_page: not mapped")
            } else if pte.flag() == PteFlag::PTE_VALID {
                Err("unmap_page: not leaf")
            } else {
                let addr = pte.addr();
                kfree(addr as *mut _);
                pte.set_unused();
                Ok(())
            }
        },
        Some(next_level) => {
            // Allocate space for page table and call map_page with next level
            if pte.is_unused() {
                return Err("unmap_page: walk");
            }
            let next_table = unsafe { &mut *(pte.addr() as *mut PageTable) };
            unmap_page(next_table, va, next_level, do_free)
        }
    }
}
