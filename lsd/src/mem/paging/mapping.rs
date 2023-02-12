use core::convert::TryInto;

use super::physical_addr::PhyscialAddress;
use super::virtual_addr::VirtualAddress;
use super::pagetable::{PageTable, PageTableAlloc};
use super::entries::{EntryFlags, Entry};

pub struct Mapper {
    root: &'static mut PageTable,
    alloc: PageTableAlloc
}

impl Mapper {
    pub fn new(root: &'static mut PageTable, alloc: PageTableAlloc) -> Self {
        Self {
            root,
            alloc
        }
    }

    //currently only supports Sv39 paging
    //No mega/giga-pages
    pub fn map(&mut self, phys: PhyscialAddress, virt: VirtualAddress, flags: EntryFlags) -> Result<(), MappingError> {
        // Invalid flag sets: 
        // W
        // XW
        if flags.contains(EntryFlags::WRITE) && !flags.contains(EntryFlags::READ) {
            return Err(MappingError::InvalidPermissions);
        }


        let sections = virt.sections();

        let ppn1_entry = self.root[sections.vpn2 as usize];
        let ppn1_ptr: *mut PageTable = match self.page_check(ppn1_entry) {
            None => return Err(MappingError::Gigapage),
            Some(ptr) => ptr
        };
        let ppn1 = unsafe {&mut *ppn1_ptr};

        if !ppn1_entry.has_flag(EntryFlags::VALID) {
            self.root[sections.vpn2 as usize].set_addr(ppn1_ptr as u64);
            self.root[sections.vpn2 as usize].add_flag(flags);
        }

        let ppn0_entry = (ppn1)[sections.vpn1.try_into().unwrap()];
        let ppn0_ptr: *mut PageTable = match self.page_check(ppn0_entry) {
            None => return Err(MappingError::Megapage),
            Some(ptr) => ptr
        };
        let ppn0 = unsafe {&mut *ppn0_ptr};

        if !ppn0_entry.has_flag(EntryFlags::VALID) {
            ppn0[sections.vpn2 as usize].set_addr(ppn0_ptr as u64);
            ppn0[sections.vpn2 as usize].add_flag(flags);
        }


        let mut entry = Entry::new(0);
        entry.set_addr(phys.as_u64() >> 12);
        entry.add_flag(flags);

        ppn0[sections.vpn0.try_into().unwrap()] = entry;

        Ok(())
    }

    pub fn page_check(&mut self, entry: Entry) -> Option<*mut PageTable> {
        // Invalid flag sets:
        // W
        // XW

        return match entry.has_flag(EntryFlags::VALID) {
            true => {
                return match entry.has_flag(EntryFlags::WRITE) && !entry.has_flag(EntryFlags::READ) {
                    true => None,
                    _ => return Some(entry.addr())
                }
            },
            false => Some(self.alloc.alloc())
        };
    }
}

#[derive(Debug)]
pub enum MappingError {
    InvalidPermissions,
    Megapage,
    Gigapage,
}