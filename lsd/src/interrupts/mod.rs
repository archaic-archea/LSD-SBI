use log::{log, Level};

#[thread_local]
static mut INT_SSCRATCH: Sscratch = Sscratch { 
    kernel_stack_top: core::ptr::null_mut(),
    kernel_thread_local: core::ptr::null_mut(),
    kernel_global_ptr: core::ptr::null_mut(),
    scratch_sp: 0
};

pub fn init() {
    use super::control_registers;
    
    unsafe {
        set_handler_fn(int_handler);
        log!(Level::Info, "Set vector of handler");
        let sie = control_registers::Sie::all() | control_registers::Sie::read();
        let sstatus = control_registers::Sstatus::read() | control_registers::Sstatus::SIE;
        //log!(Level::Debug, "SIE: {:?}, SSTATUS: {:?}", sie, sstatus);
        sie.write();
        sstatus.write();

        INT_SSCRATCH.kernel_thread_local = crate::utils::linker::__tdata_start.as_ptr().cast_mut();
        INT_SSCRATCH.kernel_global_ptr = crate::utils::linker::__global_pointer.as_ptr().cast_mut();
        INT_SSCRATCH.kernel_stack_top = crate::mem::MEM_VEC.lock().find_id("int_stack0").unwrap().base();
        let sscratch_ref = (&INT_SSCRATCH as *const Sscratch) as usize;

        core::arch::asm!(
            "csrw sscratch, {}",
            in(reg) sscratch_ref
        );

        log::info!("Interrupts enabled")
    }
}

/// Sets the trap handler address to the given function
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

#[repr(C)]
pub struct TrapFrame {
    pub sepc: usize,
    pub registers: GeneralRegisters,
}

#[repr(C)]
pub struct Sscratch {
    pub kernel_stack_top: *mut u8,
    pub kernel_thread_local: *mut u8,
    pub kernel_global_ptr: *mut u8,
    pub scratch_sp: usize,
}

// Repnops code... again... ty, Vanadinite
#[naked]
#[repr(align(4))]
pub extern "C" fn int_handler() {
    unsafe {
        core::arch::asm!(
            r#"
            // Interrupts are disabled when we enter a trap
            // Switch `t6` and `sscratch`
            csrrw t6, sscratch, t6

            // Store current stack pointer temporarily
            sd sp, 24(t6)

            // Load kernel's stack pointer
            ld sp, 0(t6)
            addi sp, sp, {TRAP_FRAME_SIZE}

            // ###############################################
            // # Begin storing userspace state in trap frame #
            // ###############################################
            sd ra, 8(sp)

            // Load and save the userspace stack pointer using
            // the now freed `ra` register
            ld ra, 24(t6)
            sd ra, 16(sp)

            // Save the other registers regularly
            sd gp, 24(sp)
            sd tp, 32(sp)
            sd t0, 40(sp)
            sd t1, 48(sp)
            sd t2, 56(sp)
            sd s0, 64(sp)
            sd s1, 72(sp)
            sd a0, 80(sp)
            sd a1, 88(sp)
            sd a2, 96(sp)
            sd a3, 104(sp)
            sd a4, 112(sp)
            sd a5, 120(sp)
            sd a6, 128(sp)
            sd a7, 136(sp)
            sd s2, 144(sp)
            sd s3, 152(sp)
            sd s4, 160(sp)
            sd s5, 168(sp)
            sd s6, 176(sp)
            sd s7, 184(sp)
            sd s8, 192(sp)
            sd s9, 200(sp)
            sd s10, 208(sp)
            sd s11, 216(sp)
            sd t3, 224(sp)
            sd t4, 232(sp)
            sd t5, 240(sp)
            ld tp, 8(t6)
            ld gp, 16(t6)

            // Swap `t6` and `sscratch` again
            csrrw t6, sscratch, t6
            sd t6, 248(sp)

            // Save `sepc`
            csrr t6, sepc
            sd t6, 0(sp)
            mv a0, sp
            csrr a1, scause
            csrr a2, stval

            // Check if floating point registers are dirty
            csrr s0, sstatus
            srli s0, s0, 13
            andi s0, s0, 3
            li s1, 3
            
            // Skip FP reg saving if they're clean
            bne s0, s1, 1f
            addi sp, sp, -264
            .attribute arch, "rv64imafdc"
            fsd f0, 0(sp)
            fsd f1, 8(sp)
            fsd f2, 16(sp)
            fsd f3, 24(sp)
            fsd f4, 32(sp)
            fsd f5, 40(sp)
            fsd f6, 48(sp)
            fsd f7, 56(sp)
            fsd f8, 64(sp)
            fsd f9, 72(sp)
            fsd f10, 80(sp)
            fsd f11, 88(sp)
            fsd f12, 96(sp)
            fsd f13, 104(sp)
            fsd f14, 112(sp)
            fsd f15, 120(sp)
            fsd f16, 128(sp)
            fsd f17, 136(sp)
            fsd f18, 144(sp)
            fsd f19, 152(sp)
            fsd f20, 160(sp)
            fsd f21, 168(sp)
            fsd f22, 176(sp)
            fsd f23, 184(sp)
            fsd f24, 192(sp)
            fsd f25, 200(sp)
            fsd f26, 208(sp)
            fsd f27, 216(sp)
            fsd f28, 224(sp)
            fsd f29, 232(sp)
            fsd f30, 240(sp)
            fsd f31, 248(sp)
            frcsr t1
            sd t1, 256(sp)
            .attribute arch, "rv64imac"
            li t1, (0b01 << 13)
            csrc sstatus, t1

            // FP registers clean
            1:
            call handler

            // Check FP register status again
            bne s0, s1, 2f

            // Restore if they were dirty
            .attribute arch, "rv64imafdc"
            fld f0, 0(sp)
            fld f1, 8(sp)
            fld f2, 16(sp)
            fld f3, 24(sp)
            fld f4, 32(sp)
            fld f5, 40(sp)
            fld f6, 48(sp)
            fld f7, 56(sp)
            fld f8, 64(sp)
            fld f9, 72(sp)
            fld f10, 80(sp)
            fld f11, 88(sp)
            fld f12, 96(sp)
            fld f13, 104(sp)
            fld f14, 112(sp)
            fld f15, 120(sp)
            fld f16, 128(sp)
            fld f17, 136(sp)
            fld f18, 144(sp)
            fld f19, 152(sp)
            fld f20, 160(sp)
            fld f21, 168(sp)
            fld f22, 176(sp)
            fld f23, 184(sp)
            fld f24, 192(sp)
            fld f25, 200(sp)
            fld f26, 208(sp)
            fld f27, 216(sp)
            fld f28, 224(sp)
            fld f29, 232(sp)
            fld f30, 240(sp)
            fld f31, 248(sp)
            ld t1, 256(sp)
            fscsr t1
            .attribute arch, "rv64imac"
            addi sp, sp, 264

            // FP registers clean
            2:

            // Restore `sepc`
            ld t6, 0(sp)
            csrw sepc, t6

            // Reenable interrupts after sret (set SPIE)
            li t6, 1 << 5
            csrs sstatus, t6
            ld ra, 8(sp)

            // Skip sp for... obvious reasons
            ld gp, 24(sp)
            ld tp, 32(sp)
            ld t0, 40(sp)
            ld t1, 48(sp)
            ld t2, 56(sp)
            ld s0, 64(sp)
            ld s1, 72(sp)
            ld a0, 80(sp)
            ld a1, 88(sp)
            ld a2, 96(sp)
            ld a3, 104(sp)
            ld a4, 112(sp)
            ld a5, 120(sp)
            ld a6, 128(sp)
            ld a7, 136(sp)
            ld s2, 144(sp)
            ld s3, 152(sp)
            ld s4, 160(sp)
            ld s5, 168(sp)
            ld s6, 176(sp)
            ld s7, 184(sp)
            ld s8, 192(sp)
            ld s9, 200(sp)
            ld s10, 208(sp)
            ld s11, 216(sp)
            ld t3, 224(sp)
            ld t4, 232(sp)
            ld t5, 240(sp)
            ld t6, 248(sp)

            // Clear any outstanding atomic reservations
            sc.d zero, zero, 0(sp)

            // Restore `sp`
            ld sp, 16(sp)
            
            // gtfo
            sret
            "#,
            TRAP_FRAME_SIZE = const { -(core::mem::size_of::<TrapFrame>() as isize) },
            options(noreturn)
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
        15 => log::error!("Store page fault"),
        _ => log::error!("Unknown exception {:b}", code)
    }

    super::hcf();
}

fn interrupt(code: u64) {
    use core::sync::atomic::Ordering;

    match code {
        1 => {
            //ipi
            let id = crate::HART_ID.load(Ordering::Relaxed);
            let mswi_base = crate::MSWI.load(Ordering::Relaxed);
            unsafe {
                *mswi_base.add(id) = 0;
            }

            log::info!("IPI occured, targeting id: {}", id);
        },
        5 => {
            //timer interrupt
            super::timing::WAIT.store(false, Ordering::Relaxed);
            log::info!("Timer interrupt");
        },
        9 => {
            //plic interrupt
            plic_int()
        },
        _ => log::error!("Error has occured, handler was called with vector: {:b}", code),
    }
}

fn plic_int() {
    let context = crate::current_context();

    let plic = unsafe {&mut *crate::plic::PLIC_REF};
    let pot_int = plic.claim(context);

    match pot_int {
        None => (),
        Some(int_claim) => {
            let int = int_claim.interrupt_id();

            match int {
                10 => {
                    uart();
                },
                _ => {
                    log::error!("Unrecognized external interrupt: {}", int);
                }
            }

            int_claim.complete();
        }
    }
}

fn uart() {
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
}

#[repr(C)]
pub struct GeneralRegisters {
    pub ra: usize, // trapframe offset 8
    pub gp: usize, // trapframe offset 16
    pub sp: usize, // trapframe offset 24
    pub tp: usize, // trapframe offset 32
    pub t0: usize, // trapframe offset 40
    pub t1: usize, // trapframe offset 48
    pub t2: usize, // trapframe offset 56
    pub s0: usize, // trapframe offset 64
    pub s1: usize, // trapframe offset 72
    pub a0: usize, // trapframe offset 80
    pub a1: usize, // trapframe offset 88
    pub a2: usize, // trapframe offset 96
    pub a3: usize, // trapframe offset 104
    pub a4: usize, // trapframe offset 112
    pub a5: usize, // trapframe offset 120
    pub a6: usize, // trapframe offset 128
    pub a7: usize, // trapframe offset 136
    pub s2: usize, // trapframe offset 144
    pub s3: usize, // trapframe offset 152
    pub s4: usize, // trapframe offset 160
    pub s5: usize, // trapframe offset 168
    pub s6: usize, // trapframe offset 176
    pub s7: usize, // trapframe offset 184
    pub s8: usize, // trapframe offset 192
    pub s9: usize, // trapframe offset 200
    pub s10: usize, // trapframe offset 208
    pub s11: usize, // trapframe offset 216
    pub t3: usize, // trapframe offset 224
    pub t4: usize, // trapframe offset 232
    pub t5: usize, // trapframe offset 240
    pub t6: usize, // trapframe offset 248
}

impl GeneralRegisters {
    pub const fn null() -> Self {
        Self { 
            ra: 0, 
            sp: 0, 
            gp: 0, 
            tp: 0, 
            t0: 0, 
            t1: 0, 
            t2: 0, 
            s0: 0, 
            s1: 0, 
            a0: 0, 
            a1: 0, 
            a2: 0, 
            a3: 0, 
            a4: 0, 
            a5: 0, 
            a6: 0, 
            a7: 0, 
            s2: 0, 
            s3: 0, 
            s4: 0, 
            s5: 0, 
            s6: 0, 
            s7: 0, 
            s8: 0, 
            s9: 0, 
            s10: 0,
            s11: 0, 
            t3: 0, 
            t4: 0, 
            t5: 0, 
            t6: 0 
        }
    }
}