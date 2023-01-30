#![feature(naked_functions)]
#![no_std]
#![no_main]

extern "C" fn kmain() -> ! {
    use log::{Level, log};
    use lsd::*;

    io::logger::init();

    unsafe {
        lsd::set_handler_fn(handler);
        log!(Level::Info, "Set vector of handler");
        let sie = interrupts::Sie::supervisor_all() | interrupts::Sie::read();
        let sstatus = interrupts::Sstatus::read() | interrupts::Sstatus::SIE;
        log!(Level::Debug, "SIE: {:?}, SSTATUS: {:?}", sie, sstatus);
        sie.write();
        sstatus.write();
    }

    sbi::timer::set_timer(20_000_000).expect("Failed to enable timer interrupt");

    hcf()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use log::{Level, log};

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