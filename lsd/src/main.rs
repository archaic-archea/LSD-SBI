#![feature(naked_functions)]
#![no_std]
#![no_main]

use log::{log, Level};

extern "C" fn kmain(_hartid: u64, devicetree_ptr: *const u8) -> ! {
    use lsd::*;

    io::logger::init();
    syscon_rs::init(devicetree_ptr);
    timing::init(devicetree_ptr);

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).expect("Failed to get fdt");
    }

    for node in fdt.all_nodes() {
        log!(Level::Info, "Node: {}", node.name);
    }
    
    let first_core = fdt.cpus().next().unwrap();
    log!(Level::Info, "Frequency: {}hz", first_core.timebase_frequency());

    timing::wait(timing::Time::Second(4));

    syscon_rs::power_off().expect("Failed to power off");

    unreachable!();
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