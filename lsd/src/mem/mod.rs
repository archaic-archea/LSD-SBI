mod linked_list;
pub mod paging;

use linked_list::LinkedListAllocator;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

// This should only be editted once, so help me if you try to for a 2nd time I am going to throw you down the stairs
#[no_mangle]
pub static mut MEMMAP: Memmap = Memmap::null();

pub fn init(devicetree_ptr: *const u8) {
    memory_map(devicetree_ptr);
    unsafe {
        let heap_data = MEMMAP.heap;
        ALLOCATOR.lock().init(heap_data.base() as usize, heap_data.length());
    }

    let free = unsafe {MEMMAP.free.length()} / 1048576;
    log::info!("Free memory: {}MiB", free);
}

pub fn memory_map(devicetree_ptr: *const u8) {
    use crate::utils::linker;

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).unwrap();
    }

    let mem = fdt.memory();
    let mem_region = mem.regions().next().unwrap();

    let mem_base = mem_region.starting_address;
    let mem_len = mem_region.size.unwrap();

    let kernel_base = unsafe {linker::KERNEL_START.as_ptr()};
    let kernel_len = unsafe {linker::KERNEL_END.as_usize() - linker::KERNEL_START.as_usize()};

    let unknown_base = mem_base;
    let unknown_len = kernel_base as usize - mem_base as usize;

    let heap_base = unsafe {kernel_base.add(kernel_len).cast_mut()};
    let heap_len = 0x4000;

    
    let stack_len = 0x100000;
    let stack_base = unsafe {align_up(heap_base.add(heap_len).add(stack_len) as usize, 16)} as *mut u8;

    let int_stack_len = 0x10000;
    let int_stack_base = unsafe {align_up(stack_base.add(stack_len).add(int_stack_len) as usize, 16)} as *mut u8;

    let free_base = unsafe {heap_base.add(heap_len)};
    let free_len = mem_len - (kernel_len + unknown_len + heap_len + stack_len + int_stack_len);

    let memmap = unsafe {&mut MEMMAP};
    memmap.mem = ConstMemRange::new(mem_base, mem_len);
    memmap._unknown = ConstMemRange::new(unknown_base, unknown_len);
    memmap.kernel = ConstMemRange::new(kernel_base, kernel_len);
    memmap.heap = MutMemRange::new(heap_base, heap_len);
    memmap.stack = MutMemRange::new(stack_base, stack_len);
    memmap.interrupt_stack = MutMemRange::new(int_stack_base, int_stack_len);
    memmap.free = MutMemRange::new(free_base, free_len);
}

unsafe impl Sync for Memmap {}
unsafe impl Send for Memmap {}

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

pub struct ConstMemRange {
    base: *const u8,
    length: usize
}

impl ConstMemRange {
    pub fn new(base: *const u8, length: usize) -> Self {
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
}

pub struct MutMemRange {
    base: *mut u8,
    length: usize
}

impl MutMemRange {
    pub fn new(base: *mut u8, length: usize) -> Self {
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
}

pub struct Memmap {
    pub mem: ConstMemRange,
    _unknown: ConstMemRange, //not sure whats here 
    kernel: ConstMemRange,
    pub heap: MutMemRange,
    pub stack: MutMemRange,
    pub interrupt_stack: MutMemRange,
    pub free: MutMemRange,
}

impl Memmap {
    pub const fn null() -> Self {
        Memmap { 
            mem: ConstMemRange::new(core::ptr::null(), 0), 
            _unknown: ConstMemRange::new(core::ptr::null(), 0), 
            kernel: ConstMemRange::new(core::ptr::null(), 0),
            heap: MutMemRange::new(core::ptr::null_mut(), 0),
            stack: MutMemRange::new(core::ptr::null_mut(), 0),
            interrupt_stack: MutMemRange::new(core::ptr::null_mut(), 0),
            free: MutMemRange::new(core::ptr::null_mut(), 0),
        }
    }
}