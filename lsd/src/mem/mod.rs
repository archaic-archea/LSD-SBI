mod linked_list;
pub mod paging;

use spin::Mutex;
use linked_list::LinkedListAllocator;

use crate::LLVec;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

// This *can* be editted more than once, but its discouraged, and I recommend its only editted when initialized
#[no_mangle]
pub static MEM_VEC: Mutex<LLVec<IDedMemRange>> = Mutex::new(LLVec::new());

pub static mut PAGING_TYPE: paging::PagingType = paging::PagingType::Sv39;

pub fn init(devicetree_ptr: *const u8) {
    memory_map(devicetree_ptr);

    let free = MEM_VEC.lock().find_id("free0").expect("No free memory").data.range.length();
    log::info!("Free memory: {}MiB", free);

    paging::init();
}

pub fn memory_map(devicetree_ptr: *const u8) {
    use crate::utils::linker;

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).unwrap();
    }

    let mem = fdt.memory();
    let mem_region = mem.regions().next().unwrap();

    let mem_base = mem_region.starting_address.cast_mut();
    let mem_len = mem_region.size.unwrap();

    let kernel_base = unsafe {linker::KERNEL_START.as_ptr().cast_mut()};
    let kernel_len = unsafe {linker::KERNEL_END.as_usize() - linker::KERNEL_START.as_usize()};

    let unknown_base = mem_base;
    let unknown_len = kernel_base as usize - mem_base as usize;

    let heap_base = unsafe {kernel_base.add(kernel_len)};
    let heap_len = 0x4000;

    unsafe {
        ALLOCATOR.lock().init(heap_base as usize, heap_len);
    }

    
    let stack_len = 0x100000;
    let stack_base = unsafe {align_up(heap_base.add(heap_len).add(stack_len) as usize, 16)} as *mut u8;

    let int_stack_len = 0x10000;
    let int_stack_base = unsafe {align_up(stack_base.add(stack_len).add(int_stack_len) as usize, 16)} as *mut u8;

    let free_base = unsafe {heap_base.add(heap_len)};
    let free_len = mem_len - (kernel_len + unknown_len + heap_len + stack_len + int_stack_len);

    let memory_range = MutMemRange::new(mem_base, mem_len);
    let unknown_range = MutMemRange::new(unknown_base, unknown_len);
    let kernel_range = MutMemRange::new(kernel_base, kernel_len);
    let heap_range = MutMemRange::new(heap_base, heap_len);
    let stack_range = MutMemRange::new(stack_base, stack_len);
    let int_stack_range = MutMemRange::new(int_stack_base, int_stack_len);
    let free_range = MutMemRange::new(free_base, free_len);

    let memory = IDedMemRange::new("mem", memory_range);
    let unknown = IDedMemRange::new("unknown", unknown_range);
    let kernel = IDedMemRange::new("kernel", kernel_range);
    let heap = IDedMemRange::new("heap0", heap_range);
    let stack = IDedMemRange::new("stack0", stack_range);
    let int_stack = IDedMemRange::new("int_stack0", int_stack_range);
    let free = IDedMemRange::new("free0", free_range);

    let mem_vec = &mut MEM_VEC.lock();
    mem_vec.initialize(memory);
    mem_vec.push(unknown);
    mem_vec.push(kernel);
    mem_vec.push(heap);
    mem_vec.push(stack);
    mem_vec.push(int_stack);
    mem_vec.push(free);
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[derive(Debug)]
pub struct ConstMemRange {
    base: *const u8,
    length: usize
}

impl ConstMemRange {
    pub const fn new(base: *const u8, length: usize) -> Self {
        Self { base, length }
    }

    pub fn base(&self) -> *const u8 {
        self.base
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn max(&self) -> *const u8 {
        unsafe {
            self.base.add(self.length)
        }
    }

    pub fn range(&self) -> core::ops::Range<u64> {
        let base = self.base() as u64;
        let max = self.max() as u64;

        base..max
    }

    pub fn size_set(&self) -> paging::PageSizeSet {
        use paging::PageSize;

        let size = self.length;
        let mut remaining_size: usize;

        let large_pages = size / PageSize::Large as usize;
        remaining_size = size % PageSize::Large as usize;

        let medium_pages = remaining_size / PageSize::Medium as usize;
        remaining_size = size % PageSize::Medium as usize;

        let mut small_pages = remaining_size / PageSize::Small as usize;
        remaining_size = size % PageSize::Small as usize;

        if remaining_size > 0 {
            small_pages += 1;
        }

        paging::PageSizeSet {
            large: large_pages,
            medium: medium_pages,
            small: small_pages
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MutMemRange {
    base: *mut u8,
    length: usize
}

impl MutMemRange {
    pub const fn new(base: *mut u8, length: usize) -> Self {
        Self { base, length }
    }

    pub fn base(&self) -> *mut u8 {
        self.base
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn max(&self) -> *mut u8 {
        unsafe {
            self.base.add(self.length)
        }
    }

    pub fn range(&self) -> core::ops::Range<u64> {
        let base = self.base() as u64;
        let max = self.max() as u64;

        base..max
    }

    pub fn size_set(&self) -> paging::PageSizeSet {
        use paging::PageSize;

        let size = self.length;
        let mut remaining_size: usize;

        let large_pages = size / PageSize::Large as usize;
        remaining_size = size % PageSize::Large as usize;

        let medium_pages = remaining_size / PageSize::Medium as usize;
        remaining_size = size % PageSize::Medium as usize;

        let mut small_pages = remaining_size / PageSize::Small as usize;
        remaining_size = size % PageSize::Small as usize;

        if remaining_size > 0 {
            small_pages += 1;
        }

        paging::PageSizeSet {
            large: large_pages,
            medium: medium_pages,
            small: small_pages
        }
    }
}

#[derive(Clone, Copy)]
pub struct IDedMemRange {
    id: [u8; 12],
    pub range: MutMemRange
}

impl IDedMemRange {
    pub fn null() -> Self {
        Self { id: [0; 12], range: MutMemRange::new(core::ptr::null_mut(), 0) }
    }

    pub fn new(id: &str, range: MutMemRange) -> Self {
        let mut new_self = Self::null();

        new_self.set_id(id);
        new_self.set_range(range);

        new_self
    }

    pub fn id(&self) -> alloc::string::String {
        let mut string = alloc::string::String::new();

        for char in self.id.iter() {
            if *char != 0 {
                string.push(*char as char);
            }
        }

        string
    }

    pub fn set_id(&mut self, id: &str) {
        let mut i = 0;

        for char in id.chars() {
            self.id[i] = char as u8;

            i += 1;
        }
    }

    pub fn set_range(&mut self, range: MutMemRange) {
        self.range = range;
    }

    pub fn base(&self) -> *mut u8 {
        self.range.base()
    }

    pub fn max(&self) -> *mut u8 {
        self.range.max()
    }

    pub fn length(&self) -> usize {
        self.range.length()
    }

    pub fn range(&self) -> core::ops::Range<u64> {
        self.range.range()
    }

    pub fn size_set(&self) -> paging::PageSizeSet {
        self.range.size_set()
    }
}

use core::fmt::{self, Write};

impl fmt::Debug for IDedMemRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_char('"')?;
        for character in self.id.iter() {
            f.write_char((*character) as char)?;
        }
        f.write_char('"')?;

        Ok(())
    }
}