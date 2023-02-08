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

pub static CHAR_BUF: spin::Mutex<[char; 64]> = spin::Mutex::new([0 as char; 64]);

use log::{log, Level};

pub mod io;
pub mod control_registers;
pub mod interrupts;
pub mod timing;
pub mod plic;
pub mod uart;
pub mod volatile;
pub mod mem;
pub mod utils;

pub fn reach_loop() {
    log::info!("Reached loop");
    loop {
        crate::wfi();
    }
}

pub fn init_tp() {
    extern {
        static __tdata_start: ();
        static __tdata_end: ();
    }

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