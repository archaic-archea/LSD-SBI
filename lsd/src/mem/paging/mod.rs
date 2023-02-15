pub mod pagetable;
pub mod physical_addr;
pub mod virtual_addr;
pub mod mapping;
pub mod entries;

const PAGE_SIZE: usize = 4096;

pub fn init() {
    use crate::mem;

    //Get memory range, and free memory range
    let mem = mem::MEM_VEC.lock().find_id("mem").unwrap().data;
    let kern = mem::MEM_VEC.lock().find_id("kernel").unwrap().data;
    let int_stack = mem::MEM_VEC.lock().find_id("int_stack0").unwrap().data;
    let free = mem::MEM_VEC.lock().find_id("free0").unwrap().data;

    //Create a new allocator for page tables
    let mut allocator = pagetable::PageTableAlloc::new(free.base());

    //Create root page table
    let table_ptr = allocator.alloc();
    let root_table = unsafe {&mut *table_ptr};

    //Create mapper for mapping memory
    let mut mapper = mapping::Mapper::new(root_table, allocator, unsafe {super::PAGING_TYPE});

    log::info!("Mapping addresses");

    let mem_range_base = mem.base() as u64;
    let mem_range_top = int_stack.base() as u64;

    let mem_range = mem_range_base..mem_range_top;

    //Loop through all non-free memory addresses to map while stepping up by 4096 each loop
    for addr in mem_range.step_by(PageSize::Small as usize) {
        let phys = physical_addr::PhyscialAddress::new(addr);
        let virt = virtual_addr::VirtualAddress::new(addr);

        //make the PTE accessed, dirty, executable, readable, writable, and valid
        use entries::EntryFlags;
        let flags = EntryFlags::ACCESSED | EntryFlags::DIRTY | EntryFlags::READ | EntryFlags::WRITE | EntryFlags::VALID;

        //log::debug!("Mapping mem: {:?} to {:?}", phys, virt);
        mapper.recursive_map(phys, virt, flags, PageSize::Small).expect("Failed to map address");
    }
    log::info!("Addresses mapped");

    log::info!("Mapping kernel");
    //Loop through all addresses to map while stepping up by 4096 each loop
    for addr in kern.range().step_by(PageSize::Small as usize) {
        let phys = physical_addr::PhyscialAddress::new(addr);
        let virt = virtual_addr::VirtualAddress::new(addr);

        //make the PTE accessed, dirty, executable, readable, writable, and valid
        use entries::EntryFlags;
        let flags = EntryFlags::ACCESSED | EntryFlags::DIRTY | EntryFlags::EXECUTE | EntryFlags::READ | EntryFlags::WRITE | EntryFlags::VALID;

        //log::debug!("Mapping mem: {:?} to {:?}", phys, virt);
        mapper.recursive_map(phys, virt, flags, PageSize::Small).expect("Failed to map address");
    }
    log::info!("Kernel mapped");

    log::info!("Mapping IO");
    let range_bot = 0;
    let range_top = 0x8000_0000;
    //Map IO
    for addr in (range_bot..range_top).step_by(PageSize::Large as usize) {
        let phys = physical_addr::PhyscialAddress::new(addr);
        let virt = virtual_addr::VirtualAddress::new(addr);

        //make the PTE accessed, dirty, readable, writable, and valid
        use entries::EntryFlags;
        let flags = EntryFlags::ACCESSED | EntryFlags::DIRTY | EntryFlags::READ | EntryFlags::WRITE | EntryFlags::VALID;

        //log::debug!("Mapping IO: {:?} to {:?}", phys, virt);
        mapper.recursive_map(phys, virt, flags, PageSize::Large).expect("Failed to map address");
    }

    log::info!("IO mapped");

    use crate::control_registers::{Satp, SatpState};

    //enable paging
    let state = SatpState::new(PagingType::Sv39, 0, unsafe {&*table_ptr}.ppn());
    log::info!("Enabling paging");
    Satp::write_state(state);
    log::info!("Paging enabled");
    log::info!("Pagetables used: {}", mapper.alloc.page_offset);
}

pub struct Page([u8; PAGE_SIZE]);

#[derive(Debug, Clone, Copy)]
pub enum PagingType {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10
}

pub enum PageSize {
    Small = 0x1000,
    Medium = 0x20_0000,
    Large = 0x4000_0000
}

pub struct PageSizeSet {
    pub large: usize,
    pub medium: usize,
    pub small: usize
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