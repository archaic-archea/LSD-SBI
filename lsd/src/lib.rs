#![no_std]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(pointer_byte_offsets)]

use log::{log, Level};

pub mod io;
pub mod control_registers;

/// Sets the interrupt vector address to the given function
pub unsafe fn set_handler_fn(f: extern "C" fn()) {
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
pub extern "C" fn handler() {
    let int_vec = interrupt_vector();
    log!(Level::Info, "Interrupt generated");

    match int_vec {
        9223372036854775813 => (),
        _ => log!(Level::Error, "Error has occured, handler was called with vector: {}", int_vec),
    }
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

pub fn power_off(fdt: fdt::Fdt) ->  Option<()> {
    let poweroff_node = fdt.find_compatible(&["syscon-poweroff"])?;
    let offset = poweroff_node.property("offset")?.as_usize()?;
    let value = poweroff_node.property("value")?.as_usize()? as u32;
    let syscon_phandle = poweroff_node.property("regmap")?.as_usize()? as u32;
    let syscon_node = fdt.find_phandle(syscon_phandle)?;
    let syscon_mmio = syscon_node.reg()?.next()?.starting_address.cast::<u32>().cast_mut();
    log!(Level::Info, "Powered off");
    unsafe {
        syscon_mmio.byte_add(offset).write_volatile(value);
    }

    None
}

pub fn reboot(fdt: fdt::Fdt) ->  Option<()> {
    let reboot = fdt.find_compatible(&["syscon-reboot"])?;
    let offset = reboot.property("offset")?.as_usize()?;
    let value = reboot.property("value")?.as_usize()? as u32;
    let syscon_phandle = reboot.property("regmap")?.as_usize()? as u32;
    let syscon_node = fdt.find_phandle(syscon_phandle)?;
    let syscon_mmio = syscon_node.reg()?.next()?.starting_address.cast::<u32>().cast_mut();
    log!(Level::Info, "Rebooting...");
    unsafe {
        syscon_mmio.byte_add(offset).write_volatile(value);
    }

    None
}