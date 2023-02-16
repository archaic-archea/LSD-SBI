use super::{entries::Entry, physical_addr};

#[derive(Debug)]
#[repr(align(0x1000))]
pub struct PageTable(pub [Entry; 512]);

impl PageTable {
    pub fn ppn(&self) -> physical_addr::Ppn {
        let addr = self as *const PageTable as u64;

        physical_addr::Ppn::from_phys(physical_addr::PhyscialAddress::new(addr))
    }
}

impl core::ops::Index<usize> for PageTable {
    type Output = Entry;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl core::ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, idx: usize) -> &mut Entry {
        &mut self.0[idx]
    }
}

pub struct PageTableAlloc {
    pub pages_used: usize
}

use alloc::alloc::{dealloc, alloc, Layout};

impl PageTableAlloc {
    pub fn new() -> Self {
        Self { pages_used: 0 }
    }

    pub fn alloc(&mut self) -> *mut PageTable {
        unsafe {
            let ptr = alloc(Layout::new::<PageTable>());

            ptr as *mut PageTable
        }
    }

    pub fn dealloc(&self, page_table: *mut PageTable) {
        unsafe {
            dealloc(page_table as *mut u8, Layout::new::<PageTable>())
        }
    }
}