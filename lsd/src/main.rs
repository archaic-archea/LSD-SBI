#![feature(naked_functions)]
#![no_std]
#![no_main]

use log::{log, Level};

extern "C" fn kmain(hartid: usize, devicetree_ptr: *const u8) -> ! {
    use lsd::*;

    io::logger::init();
    syscon_rs::init(devicetree_ptr);
    timing::init(devicetree_ptr);
    interrupts::init();

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).expect("Failed to get fdt");
    }

    let uart_node = fdt.find_compatible(uart::Uart16550::compatible()).expect("Failed to find Uart");
    let uart_int = uart_node.property("interrupts").unwrap().as_usize().unwrap();
    let uart_reg = uart_node.reg().unwrap().next().unwrap();
    let uart = unsafe {&*(uart_reg.starting_address.cast_mut() as *mut uart::Uart16550)};

    uart.init();
    
    log!(Level::Info, "UART initialized");

    let plic_node = fdt.find_compatible(plic::Plic::compatible()).expect("Failed to find plic");
    let plic_region = plic_node.reg().expect("No plic region").next().unwrap();
    let plic_ref = plic::PlicRefer::new(plic_region.starting_address);

    let context = hartid * 2 + 1;

    plic_ref.init(11, context);
    plic_ref.set_priority(uart_int, 7);
    plic_ref.threshold_and_claim(context, 0);
    plic_ref.enable_int(context, uart_int);
    log!(Level::Info, "PLIC initialized");

    uart.set_int();
    log::info!("UART interrupts set");
    
    timing::wait(timing::Time::Second(8));
    syscon_rs::power_off();
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