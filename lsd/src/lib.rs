#![no_std]
#![feature(fn_align)]
#![feature(naked_functions)]

pub mod io;
pub mod interrupts;

/// Sets the interrupt vector address to the given function
pub unsafe fn set_handler_fn(f: extern "C" fn() -> !) {
    core::arch::asm!("csrw stvec, {}", in(reg) f);
}

pub fn interrupt_vector() -> u64 {
    let x: u64;

    unsafe {
        core::arch::asm!(
            "csrr {}, scause",
            out(reg) x
        );
    }

    x
}

#[repr(align(4))]
pub extern "C" fn handler() -> ! {
    use log::{Level, log};

    log!(Level::Error, "Error has occured, handler was called with vector: {}", interrupt_vector());

    hcf();
}

pub fn time_int() {
    let time: u64;
    unsafe {
        core::arch::asm!(
            "rdtime {}",
            out(reg) time
        );
    }
    sbi::timer::set_timer(time + 10_000).expect("Error occurred");
}

pub fn hcf() -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") };
    }
}

pub fn fail() {
    unsafe {
        core::arch::asm!("unimp");
    }
}

pub fn core_bootstrap() -> ! {
    use log::{Level, log};

    log!(Level::Info, "Core started");

    hcf();
}