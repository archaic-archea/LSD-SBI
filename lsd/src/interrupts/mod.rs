use log::{log, Level};

pub fn init() {
    use super::control_registers;
    
    unsafe {
        set_handler_fn(handler);
        log!(Level::Info, "Set vector of handler");
        let sie = control_registers::Sie::all() | control_registers::Sie::read();
        let sstatus = control_registers::Sstatus::read() | control_registers::Sstatus::SIE;
        log!(Level::Debug, "SIE: {:?}, SSTATUS: {:?}", sie, sstatus);
        sie.write();
        sstatus.write();
    }
}

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