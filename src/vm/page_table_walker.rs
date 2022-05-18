use super::addr::VirtAddr;
use super::page_table::{PageTable, PageTableEntry, PageTableLevel};


pub struct PageTableWalkerMut<'a, Extra> {
    pub page_table: &'a mut PageTable,
    pub va: VirtAddr,
    pub level: PageTableLevel,
    pub extra: Extra,
}

pub trait PageTableVisitor {
    type Output : core::ops::Try;
    fn is_valid_va(&self, va: VirtAddr) -> bool;
    fn leaf(&mut self, pte: &mut PageTableEntry) -> Self::Output;
    fn nonleaf(&mut self, pte: &mut PageTableEntry) -> Self::Output;
}

impl<'a, Extra: PageTableVisitor> PageTableWalkerMut<'a, Extra> {
    pub fn new(page_table: &'a mut PageTable, va: VirtAddr, level: PageTableLevel, extra: Extra) -> Option<Self> {
        extra.is_valid_va(va).then(
            move || Self {
                page_table,
                va,
                level,
                extra
            }
        )
    }
    pub fn visit_mut(&mut self) -> Extra::Output {
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
