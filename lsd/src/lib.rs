#![no_std]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(pointer_byte_offsets)]

use log::{log, Level};

pub mod io;
pub mod control_registers;
pub mod power;
pub mod interrupts;
pub mod timing;

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