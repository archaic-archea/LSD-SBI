use log::{log, Level};

pub fn init() {
    use super::control_registers;
    
    unsafe {
        set_handler_fn(int_handler);
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

pub fn interrupt_vector() -> (bool, u64) {
    let x: u64;

    unsafe {
        core::arch::asm!(
            "csrr {}, scause",
            out(reg) x
        );
    }

    let mask =0x7FFFFFFFFFFFFFFF;

    ((x & (!mask)) == 0, x & mask)
}

#[no_mangle]
#[repr(align(4))]
pub extern "C" fn handler() {
    let int_vec = interrupt_vector();

    match int_vec {
        (true, code) => exception(code),
        (false, code) => interrupt(code)
    }
}

#[naked]
#[repr(align(4))]
pub extern "C" fn int_handler() {
    unsafe {
        core::arch::asm!(
            "
                // We may have trapped from userspace. If so, we need
                // to switch to the kernel stack and global pointer.
                //
                // While executing in userspace, `sscratch` contains the
                // per-cpu pointer for this cpu. While executing in kernelspace,
                // it contains zero.
                csrrw   tp, sscratch, tp
                beqz    tp, 1f
                // We've come from userspace, swap to our stack
                sd      sp, 0x10(tp)            // store user stack pointer
                csrrw   sp, sscratch, zero      // get user tp and set sscratch to zero
                sd      sp, 0x18(tp)            //      store user tp
                ld      sp, 0x08(tp)            // load kernel stack
                j       2f
                
                1:
                    // We've come from the kernel, just swap back `tp`.
                    csrrw   tp, sscratch, tp
                2:
                    call handler
                    sret
            ", options(noreturn)
        )
    }
}

fn exception(code: u64) {
    match code {
        0 => log::error!("Instruction address misaligned"),
        1 => log::error!("Instruction access fault"),
        2 => log::error!("Illegal instruction"),
        3 => log::error!("Breakpoint"),
        4 => log::error!("Load address misaligned"),
        5 => log::error!("Load access fault"),
        6 => log::error!("Store/AMO address misaligned"),
        7 => log::error!("Store/AMO access fault"),
        _ => log::error!("Unknown exception {:b}", code)
    }

    super::hcf();
}

fn interrupt(code: u64) {
    match code {
        5 => unsafe {
            super::timing::WAIT = false;
        },
        9 => plic_int(),
        _ => log::error!("Error has occured, handler was called with vector: {:b}", code),
    }
}

fn plic_int() {
    unsafe {
        use crate::plic;
        let int = plic::PLIC_REF.next(crate::current_context());
        
        if let Some(interrupt) = int {
            match interrupt {
                10 => {
                    let my_uart = crate::uart::Uart16550::new(0x1000_0000 as *const u8);
                    
                    let character = my_uart.read();
                    match character {
                        8 => {
                            my_uart.write(8);
                            my_uart.write(b' ');
                            my_uart.write(8);
                        },
                        10 | 13 => {
                            my_uart.write(b'\n');
                        },
                        _ => {
                            crate::log_print!("{}", character as char);
                        },
                    }
                },
                _ => {
                    log::error!("Unrecognized external interrupt: {}", interrupt);
                }
            }

            plic::PLIC_REF.claim(crate::current_context(), interrupt);
        }
    }
}