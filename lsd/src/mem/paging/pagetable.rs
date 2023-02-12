use super::{entries::Entry, physical_addr};

pub struct PageTable([Entry; 512]);

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
    base: *mut PageTable,
    page_offset: usize
}

impl PageTableAlloc {
    pub fn new(base: *mut u8) -> Self {
        Self { 
            base: base as *mut PageTable, 
            page_offset: 0
        }
    }
    pub fn alloc(&mut self) -> *mut PageTable {
        unsafe {
            self.page_offset += 1;

            let ptr = self.base.add(self.page_offset - 1);
            
            for i in 0..4096 {
                let byte_ptr = ptr as *mut u8;
                *byte_ptr.offset(i) = 0;
            }

            ptr
        }
    }

    pub fn dealloc(&self) {
        panic!("Dealloc shouldnt be called on permanent page tables")
    }
}