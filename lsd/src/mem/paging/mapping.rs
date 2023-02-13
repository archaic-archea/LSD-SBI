use core::convert::TryInto;

use super::{PagingType, PageSize};
use super::physical_addr::PhyscialAddress;
use super::virtual_addr::VirtualAddress;
use super::pagetable::{PageTable, PageTableAlloc};
use super::entries::{EntryFlags, Entry};

pub struct Mapper {
    root: &'static mut PageTable,
    alloc: PageTableAlloc,
    paging_type: super::PagingType
}

impl Mapper {
    pub fn new(root: &'static mut PageTable, alloc: PageTableAlloc, paging_type: super::PagingType) -> Self {
        Self {
            root,
            alloc,
            paging_type
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
            self.root[sections.vpn2 as usize].add_flag(EntryFlags::VALID);
        }


        let ppn0_entry = (ppn1)[sections.vpn1.try_into().unwrap()];
        let ppn0_ptr: *mut PageTable = match self.page_check(ppn0_entry) {
            None => return Err(MappingError::Megapage),
            Some(ptr) => ptr
        };
        let ppn0 = unsafe {&mut *ppn0_ptr};

        if !ppn0_entry.has_flag(EntryFlags::VALID) {
            ppn1[sections.vpn1 as usize].set_addr(ppn0_ptr as u64);
            ppn1[sections.vpn1 as usize].add_flag(EntryFlags::VALID);
        }

        
        let mut entry = Entry::new(0);
        entry.set_addr(phys.as_u64());
        entry.add_flag(flags);

        ppn0[sections.vpn0.try_into().unwrap()] = entry;

        Ok(())
    }

    pub fn recursive_map(&mut self, phys: PhyscialAddress, virt: VirtualAddress, flags: EntryFlags, page_size: PageSize) -> Result<(), MappingError> {
        // Invalid flag sets: 
        // W
        // XW
        if flags.contains(EntryFlags::WRITE) && !flags.contains(EntryFlags::READ) {
            return Err(MappingError::InvalidPermissions);
        }

        let lo_depth = match self.paging_type {
            PagingType::Sv39 => 2,
            PagingType::Sv48 => 1,
            paging_type => return Err(MappingError::UnsupportedPagingType(paging_type))
        };

        let hi_depth = match page_size {
            PageSize::Small => 4,
            PageSize::Medium => 3,
            PageSize::Large => 2
        };

        let sections = virt.sections();

        let mut src_table = self.root as *mut PageTable;
        let mut ppn_entry: Entry;
        let mut ppn_ptr: *mut PageTable;
        let mut ppn: &mut PageTable;

        unsafe {
            for i in lo_depth..hi_depth {
                ppn_entry = (*src_table)[sections[i] as usize];
                ppn_ptr = match self.page_check(ppn_entry) {
                    None => return Err(MappingError::Unknown),
                    Some(ptr) => ptr
                };
                ppn = &mut *ppn_ptr;

                if !ppn_entry.has_flag(EntryFlags::VALID) {
                    (*src_table)[sections[i] as usize].set_addr(ppn_ptr as u64);
                    (*src_table)[sections[i] as usize].add_flag(EntryFlags::VALID);
                }

                src_table = ppn;
            }
        }
        
        let mut entry = Entry::new(0);
        entry.set_addr(phys.as_u64());
        entry.add_flag(flags);

        unsafe {(*src_table)[sections.vpn0.try_into().unwrap()] = entry};

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
    Unknown,
    InvalidPermissions,
    Megapage,
    Gigapage,
    UnsupportedPagingType(PagingType)
}