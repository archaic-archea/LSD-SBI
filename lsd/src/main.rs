#![feature(naked_functions)]
#![no_std]
#![no_main]

use log::{log, Level};

extern "C" fn kmain() -> ! {
    use lsd::*;

    let devicetree_ptr: *const u8;
    unsafe {core::arch::asm!(
        "mv {}, a1",
        out(reg) devicetree_ptr
    );}

    io::logger::init();

    log!(Level::Debug, "Found devicetree ptr {:?}", devicetree_ptr);

    unsafe {
        let fdt = fdt::Fdt::from_ptr(devicetree_ptr).expect("Failed to get device tree");

        lsd::set_handler_fn(handler);
        log!(Level::Info, "Set vector of handler");
        let sie = control_registers::Sie::all() | control_registers::Sie::read();
        let sstatus = control_registers::Sstatus::read() | control_registers::Sstatus::SIE;
        log!(Level::Debug, "SIE: {:?}, SSTATUS: {:?}", sie, sstatus);
        sie.write();
        sstatus.write();
        
        core::arch::asm!("ecall");
    }

    hcf()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log!(Level::Error, "{}", info);
    lsd::hcf()
}


#[naked]
#[no_mangle]
#[link_section = ".init.boot"]
unsafe extern "C" fn _boot() -> ! {
    #[rustfmt::skip]
    core::arch::asm!("
        csrw sie, zero
        csrci sstatus, 2
        
        .option push
        .option norelax
        lla gp, __global_pointer$
        .option pop

        lla sp, __tmp_stack_top

        lla t0, __bss_start
        lla t1, __bss_end

        1:
            beq t0, t1, 2f
            sd zero, (t0)
            addi t0, t0, 8
            j 1b

        2:
            j {}
    ", sym kmain, options(noreturn));
}