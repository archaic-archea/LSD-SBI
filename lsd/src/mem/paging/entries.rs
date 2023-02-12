bitflags::bitflags! {
    pub struct Entry: u64 {
        const V = 1 << 0; // Valid
        const R = 1 << 1; // Read
        const W = 1 << 2; // Write
        const X = 1 << 3; // Execute
        const U = 1 << 4; // User accessible
        const G = 1 << 5; // Global
        const A = 1 << 6; // Accessed
        const D = 1 << 7; // Dirty

        const RSW = 0b11 << 8; // Unknown
        
        const ADDR_MASK = 0xfff_ffff_ffff;

        const _RESERVED = 0b1111111 << 54; // Reserved

        const PBMT = 0b11 << 61; // Unknown

        const N = 0b1 << 63; // Unknown
    }
}

impl Entry {
    pub fn new(bits: u64) -> Self {
        unsafe {
            Self::from_bits_unchecked(bits)
        }
    }

    pub fn table(&self) -> &'static mut super::pagetable::PageTable {
        use super::physical_addr::PhyscialAddress;

        let ppn = (self.bits() >> 10) & Self::ADDR_MASK.bits();
        let address = PhyscialAddress::new(ppn << 12);

        unsafe {
            let ptr = address.to_virt().to_ptr();

            &mut *ptr
        }
    }

    pub fn add_flag(&mut self, flag: EntryFlags) {
        self.bits |= flag.bits();
    }

    pub fn has_flag(&self, flag: EntryFlags) -> bool {
        let bits = self.bits();

        bits & flag.bits() > 0
    }

    pub fn addr(&self) -> *mut super::pagetable::PageTable {
        let bits = self.bits();
        let addr = (bits >> 10) & Self::ADDR_MASK.bits();
        let ptr = (addr << 12) as *mut super::pagetable::PageTable;

        ptr
    }
}

bitflags::bitflags! {
    pub struct EntryFlags: u64 {
        const VALID = 1 << 0;
        const READ = 1 << 1;
        const WRITE = 1 << 2; 
        const EXECUTE = 1 << 3; 
        const USER_ACCESSIBLE = 1 << 4;
        const GLOBAL = 1 << 5;
        const ACCESSED = 1 << 6; 
        const DIRTY = 1 << 7;
    }
}