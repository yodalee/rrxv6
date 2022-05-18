use rv64::csr::satp::{Satp, SatpMode};
use rv64::asm::sfence_vma;

use crate::vm::page_table::{PageTable, PageTableLevel, PageTableEntry};
use crate::vm::addr::{VirtAddr, PhysAddr, align_up, align_down};
use crate::vm::page_flag::PteFlag;
use crate::riscv::{PAGESIZE, MAXVA};
use crate::memorylayout::{UART0, PLIC_BASE, TRAMPOLINE, TRAPFRAME, KERNELBASE, PHYSTOP, kstack};
use crate::kalloc::{kalloc, kfree};
use crate::param::NPROC;
use crate::proc::Proc;

use core::ptr::{NonNull, copy, write_bytes};
use core::slice::from_raw_parts;
use core::cmp;

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

struct MapWalkerMut<'a, Extra> {
    page_table: &'a mut PageTable,
    va: VirtAddr,
    level: PageTableLevel,
    extra: Extra,
}

trait PageTableVisitor {
    type Output : core::ops::Try;
    fn check_va(&mut self, va: VirtAddr) -> Self::Output;
    fn leaf(&mut self, pte: &mut PageTableEntry) -> Self::Output;
    fn nonleaf(&mut self, pte: &mut PageTableEntry) -> Self::Output;
}

impl<Extra: PageTableVisitor> MapWalkerMut<'_, Extra> {
    fn visit_mut(&mut self) -> Extra::Output {
        let _ = self.extra.check_va(self.va)?;
        let index = self.va.get_index(self.level);
        let pte = &mut self.page_table[index];

        match self.level.next_level() {
            None => {
                self.extra.leaf(pte)
            }
            Some(next_level) => {
                let _ = self.extra.nonleaf(pte)?;

                let next_table = unsafe { &mut *(pte.addr() as *mut PageTable) };
                self.page_table = next_table;
                self.level = next_level;
                self.visit_mut()
            }
        }
    }
}

struct PageMapper {
    pa: PhysAddr,
    perm: PteFlag
}

impl PageTableVisitor for PageMapper {
    type Output = Result<(), &'static str>;
    fn check_va(&mut self, va: VirtAddr) -> Self::Output {
        if va >= VirtAddr::new(MAXVA) {
            return Err("map_page: virtual address over MAX address")
        }
        Ok(())
    }

    fn leaf(&mut self, pte: &mut PageTableEntry) -> Self::Output {
        if pte.is_unused() {
            pte.set_addr(self.pa.as_pte(), self.perm | PteFlag::PTE_VALID);
            Ok(())
        } else {
            Err("map_page: remap")
        }
    }

    fn nonleaf(&mut self, pte: &mut PageTableEntry) -> Self::Output {
        if pte.is_unused() {
            let ptr = kalloc();
            if ptr == 0 as *mut u8 {
                return Err("kalloc failed in map_page");
            }
            let addr = PhysAddr::new(ptr as *const _ as u64);
            pte.set_addr(addr.as_pte(), PteFlag::PTE_VALID);
        }
        Ok(())
    }
}

struct PageUnmapper {
    do_free: bool
}

impl PageTableVisitor for PageUnmapper {
    type Output = Result<(), &'static str>;
    fn check_va(&mut self, va: VirtAddr) -> Self::Output {
        if va >= VirtAddr::new(MAXVA) {
            return Err("unmap_page: virtual address over MAX address")
        }
        Ok(())
    }

    fn leaf(&mut self, pte: &mut PageTableEntry) -> Self::Output {
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
    }

    fn nonleaf(&mut self, pte: &mut PageTableEntry) -> Self::Output {
        if pte.is_unused() {
            return Err("unmap_page: walk");
        }
        Ok(())
    }
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
        let mapper = PageMapper { pa, perm };
        let mut walker = MapWalkerMut {
            page_table,
            va: page_addr,
            level: PageTableLevel::Two,
            extra: mapper
        };
        walker.visit_mut()?;

        if page_addr == va_end {
            break;
        }
        page_addr += PAGESIZE;
        pa += PAGESIZE;
    }

    Ok(())
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
        unmap_pages(page_table, VirtAddr::new(TRAMPOLINE), 1, false).and(
        unmap_pages(page_table, VirtAddr::new(TRAPFRAME), 1, false)).and(
        unmap_free(page_table, proc.memory_size))
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
        let unmapper = PageUnmapper { do_free };
        let mut walker = MapWalkerMut {
            page_table,
            va: addr,
            level: PageTableLevel::Two,
            extra: unmapper,
        };
        walker.visit_mut()?;

        addr += PAGESIZE;
    }

    Ok(())
}

fn free_pagetable(page_table: &mut PageTable, level: PageTableLevel) -> Result<(), &'static str> {
    for i in 0..512 {
        let pte = &mut page_table[i];
        let flag = pte.flag();
        if flag.contains(PteFlag::PTE_VALID) {
            if !flag.intersects(PteFlag::PTE_READ | PteFlag::PTE_WRITE | PteFlag::PTE_EXEC) {
                let next_table = unsafe { &mut *(pte.addr() as *mut PageTable) };
                free_pagetable(next_table, level.next_level().unwrap())?;
                pte.set_unused();
            } else {
                return Err("free_pagetable: leaf");
            }
        }
    }
    kfree(page_table as *mut PageTable as *mut _);
    Ok(())
}

fn unmap_free(pagetable: &mut PageTable, size: u64) -> Result<(), &'static str> {
    if size > 0 {
        let va = VirtAddr::new(0);
        let npages = align_up(size, PAGESIZE) / PAGESIZE;
        unmap_pages(pagetable, va, npages, true)?;
    }
    free_pagetable(pagetable, PageTableLevel::Two)?;
    Ok(())
}

/// Look up a virtual address, return Option physical address,
/// Can only be used to look up user pages.
fn map_addr(page_table: &PageTable, va: VirtAddr) -> Option<PhysAddr> {
    if va >= VirtAddr::new(MAXVA) {
        return None
    }
    map_addr_recur(page_table, va, PageTableLevel::Two)
}

fn map_addr_recur(page_table: &PageTable, va: VirtAddr, level: PageTableLevel) -> Option<PhysAddr> {
    let index = va.get_index(level);
    let pte = &page_table[index];
    match level.next_level() {
        None => {
            let flag = pte.flag();
            if !flag.contains(PteFlag::PTE_VALID | PteFlag::PTE_USER) {
                return None
            }
            Some(PhysAddr::new(pte.addr()))
        },
        Some(next_level) => {
            let flag = pte.flag();
            if !flag.contains(PteFlag::PTE_VALID) {
                return None;
            }
            let next_table = unsafe { &*(pte.addr() as *const PageTable) };
            map_addr_recur(next_table, va, next_level)
        }
    }
}

pub fn copy_in_str(page_table: &mut PageTable, addr: u64, buf: &mut [u8]) -> Option<u64> {
    let max_len = buf.len();
    let base = align_down(addr, PAGESIZE);
    let offset = addr - base;
    let va = VirtAddr::new(base);
    let pa = map_addr(page_table, va)?;
    let n = cmp::min(max_len, (PAGESIZE - offset) as usize);

    let addr = (pa + offset).as_u64() as *const _;
    unsafe {
        let slice: &[u8] = from_raw_parts::<u8>(addr, n);
        let len = slice
            .iter()
            .take_while(|c| **c != 0)
            .zip(buf.iter_mut())
            .map(|(a, b)| *b = *a)
            .count();
        Some(len as u64)
    }
}
