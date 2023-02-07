use spin::Mutex;
mod linked_list;

use linked_list::LinkedListAllocator;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub static MEMMAP: Mutex<Memmap> = Mutex::new(Memmap::null());

pub fn init(devicetree_ptr: *const u8) {
    memory_map(devicetree_ptr);
    unsafe {
        let heap_data = MEMMAP.lock().heap;
        ALLOCATOR.lock().init(heap_data.0 as usize, heap_data.1);
    }
}

pub struct Memmap {
    pub mem: (*const u8, usize),
    _unknown: (*const u8, usize), //not sure whats here 
    kernel: (*const u8, usize),
    pub heap: (*mut u8, usize),
    pub free: (*mut u8, usize),
}

impl Memmap {
    pub const fn null() -> Self {
        Memmap { 
            mem: (core::ptr::null(), 0), 
            _unknown: (core::ptr::null(), 0), 
            kernel: (core::ptr::null(), 0),
            heap: (core::ptr::null_mut(), 0),
            free: (core::ptr::null_mut(), 0),
        }
    }
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

    let free_base = unsafe {heap_base.add(heap_len)};
    let free_len = mem_len - (kernel_len + unknown_len + heap_len);

    let mut memmap = MEMMAP.lock();
    memmap.mem = (mem_base, mem_len);
    memmap._unknown = (unknown_base, unknown_len);
    memmap.kernel = (kernel_base, kernel_len);
    memmap.heap = (heap_base, heap_len);
    memmap.free = (free_base, free_len);
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