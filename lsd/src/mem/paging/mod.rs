pub mod pagetable;
pub mod physical_addr;
pub mod virtual_addr;
pub mod mapping;
pub mod entries;

const PAGE_SIZE: usize = 4096;

pub struct Page([u8; PAGE_SIZE]);

#[derive(Debug)]
pub enum PagingType {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10
}

impl PagingType {
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "riscv,sv39" => Some(Self::Sv39),
            "riscv,sv48" => Some(Self::Sv48),
            "riscv,sv57" => Some(Self::Sv57),
            _ => None
        }
    }

    pub fn from_usize(id: usize) -> Self {
        match id {
            8 => Self::Sv39,
            9 => Self::Sv48,
            10 => Self::Sv57,
            _ => Self::Bare
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            Self::Sv39 => 8,
            Self::Sv48 => 9,
            Self::Sv57 => 10,
            _ => 0
        }
    }
}

pub fn init() {
    //Get memory range, and free memory range
    let mem = unsafe {crate::mem::MEMMAP.mem};
    let free = unsafe {crate::mem::MEMMAP.free};

    //Create a new allocator for page tables
    let mut allocator = pagetable::PageTableAlloc::new(free.0);

    //Create root page table
    let table_ptr = allocator.alloc();
    let root_table = unsafe {&mut *table_ptr};

    //Create mapper for mapping memory
    let mut mapper = mapping::Mapper::new(root_table, allocator);

    //Get range top and bottom for use in mapping
    let range_bot = mem.0 as u64;
    let range_top = range_bot + 0x10000000;

    log::info!("Mapping addresses");

    let virt_base = virtual_addr::VirtualAddress::new(range_bot);

    log::info!("Sections: {:#?}", virt_base.sections());

    //Loop through all addresses to map while stepping up by 4096 each loop
    for addr in (range_bot..range_top).step_by(0x1000) {
        let phys = physical_addr::PhyscialAddress::new(addr);
        let virt = virtual_addr::VirtualAddress::new(addr);

        //make the PTE accessed, dirty, executable, readable, writable, and valid
        use entries::EntryFlags;
        let flags = EntryFlags::ACCESSED | EntryFlags::DIRTY | EntryFlags::EXECUTE | EntryFlags::READ | EntryFlags::WRITE | EntryFlags::VALID;

        //log::debug!("Mapping {:?} to {:?}", phys, virt);
        mapper.map(phys, virt, flags).expect("Failed to map address");
    }

    log::info!("Addresses mapped");

    for entry_num in 0..512 {
        let root = unsafe {&*table_ptr};
        let entry = root[entry_num];

        match entry.has_flag(entries::EntryFlags::VALID) {
            true => log::debug!("Found entry: {:#?}", entry),
            _ => ()
        }
    }

    use crate::control_registers::{Satp, SatpState};

    //enable paging
    let state = SatpState::new(PagingType::Sv39, 0, unsafe {&*table_ptr}.ppn());
    log::info!("enabling paging with bits 0b{:b}", state.bits());
    Satp::write_state(state);
}
