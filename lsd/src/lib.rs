#![no_std]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(thread_local)]
#![feature(pointer_byte_offsets)]
#![feature(const_mut_refs)]
#![feature(asm_const)]

extern crate alloc;

use core::sync::atomic;

#[thread_local]
pub static HART_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

use log::{log, Level};

pub mod io;
pub mod control_registers;
pub mod interrupts;
pub mod timing;
pub mod volatile;
pub mod mem;
pub mod utils;
pub mod drivers;

pub use drivers::*;

pub fn reach_loop(msg: &str) {
    log::info!("{}", msg);
    loop {
        crate::wfi();
    }
}

pub fn init_tp() {
    use utils::linker::{__tdata_start, __tdata_end};

    let start = unsafe { core::ptr::addr_of!(__tdata_start).cast_mut() };
    let _end = unsafe { core::ptr::addr_of!(__tdata_end).cast_mut() };

    unsafe {
        core::arch::asm!(
            "mv tp, {}",
            in(reg) start
        )
    }
}

pub fn hcf() -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") };
    }
}

pub fn wfi() {
    unsafe {
        core::arch::asm!("wfi");
    }
}

pub fn fail() {
    unsafe {
        core::arch::asm!("unimp");
    }
}

pub fn core_bootstrap() -> ! {
    log!(Level::Info, "Core started");

    hcf();
}

pub trait Compat {
    fn compatible() -> &'static [&'static str];
}

pub fn current_context() -> usize {
    #[cfg(not(feature = "platform.sifive_u"))]
    return 1 + 2 * crate::HART_ID.load(atomic::Ordering::Relaxed);
}

pub fn context(id: usize) -> usize {
    #[cfg(not(feature = "platform.sifive_u"))]
    return 1 + 2 * id;
}

//Linked List Vector, used for static vectors
pub struct LLVec<T> 
where 
    T: 'static,
    T: Sized {
    start: Option<*mut LLVecEntry::<T>>,
    length: usize
}

#[derive(Debug)]
pub struct LLVecEntry<T> 
where 
    T: 'static,
    T: Sized {
    data: T,
    next: Option<*mut Self>
}

impl<T> LLVec<T> {
    pub const fn new() -> Self {
        Self { start: None, length: 0 }
    }

    pub fn initialize(&mut self, data: T) {
        use alloc::alloc;

        unsafe {
            let new_entry_ptr = alloc::alloc(alloc::Layout::new::<LLVecEntry<T>>()) as *mut LLVecEntry<T>;
            *new_entry_ptr = LLVecEntry::new(data);
            self.start = Some(&mut *new_entry_ptr);
        }

        self.length += 1;
    }

    pub fn push(&mut self, data: T) {
        use alloc::alloc;

        unsafe {
            let len = self.length;

            let new_entry_ptr = alloc::alloc(alloc::Layout::new::<LLVecEntry<T>>()) as *mut LLVecEntry<T>;
            *new_entry_ptr = LLVecEntry::new(data);
            self[len - 1].next = Some(&mut *new_entry_ptr);
        }

        self.length += 1;
    }

    pub fn remove(&mut self, index: usize) {
        use alloc::alloc;

        if index == 0 {
            panic!("Cannot remove the first entry");
        } else if index == self.length - 1 {
            panic!("Cannot remove end(yet)");
        } else if index >= self.length {
            panic!("Over index of LLVec");
        }

        let previous_entry = &mut self[index - 1];

        unsafe {
            let current_entry = &*previous_entry.next.unwrap();

            let next_entry = current_entry.next.unwrap();

            previous_entry.next = Some(next_entry);

            let current_entry_ptr = current_entry as *const LLVecEntry<T>;

            alloc::dealloc(current_entry_ptr.cast_mut() as *mut u8, alloc::Layout::new::<LLVecEntry<T>>())
        }
    }
}

use core::ops::{Index, IndexMut};

impl<T> Index<usize> for LLVec<T> 
where 
    T: 'static {
    type Output = LLVecEntry<T>;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.length {
            panic!("Indexing farther than length allows");
        }

        let mut src: &*mut LLVecEntry<T> = match &self.start {
            None => panic!("Vector not initialized"),
            Some(src) => src
        };

        unsafe {
            for _i in 0..index {
                src = match &(**src).next {
                    None => panic!("Invalid vec entry"),
                    Some(src) => src
                };
            }

            &mut **src
        }
    }
}

impl<T> IndexMut<usize> for LLVec<T> 
where 
    T: 'static {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.length {
            panic!("Indexing farther than length allows");
        }

        let mut src: &*mut LLVecEntry<T> = match &self.start {
            None => panic!("Vector not initialized"),
            Some(src) => src
        };

        unsafe {
            for _i in 0..index {
                src = match &(**src).next {
                    None => panic!("Invalid vec entry"),
                    Some(src) => src
                };
            }

            &mut **src
        }
    }
}

impl<T> LLVecEntry<T> 
where
    T: 'static,
    T: Sized {
    pub fn new(data: T) -> Self {
        Self { data, next: None }
    }

    pub fn read(&self) -> &T {
        &self.data
    }

    pub fn write(&mut self, data: T) {
        self.data = data
    }
}

use mem::IDedMemRange;

impl LLVec<IDedMemRange> {
    pub fn find_id(&self, id: &str) -> Option<&LLVecEntry<IDedMemRange>> {
        for entry_num in 0..self.length - 1 {
            if self[entry_num].read().id() == id {
                return Some(&self[entry_num]);
            }
        }

        None
    }
}

unsafe impl<T> Send for LLVec<T> {}
unsafe impl<T> Sync for LLVec<T> {}