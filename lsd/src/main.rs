#![feature(naked_functions)]
#![feature(layout_for_ptr)]
#![feature(pointer_is_aligned)]
#![no_std]
#![no_main]

use log::{log, Level};

extern crate alloc;

extern "C" fn kmain(hartid: usize, devicetree_ptr: *const u8) -> ! {
    use lsd::*;

    init_tp();
    io::logger::init();
    syscon_rs::init(devicetree_ptr);
    timing::init(devicetree_ptr);
    interrupts::init();
    mem::init(devicetree_ptr);

    let memmap = mem::MEMMAP.lock();

    unsafe {
        let stack_ptr = memmap.stack.0;

        let alignment = stack_ptr.is_aligned_to(16);
        log::info!("aligned: {}", alignment);
        core::arch::asm!(
            "mv sp, {}",
            in(reg) stack_ptr
        );
    }

    plic::init(devicetree_ptr, current_context());


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

    let plic_ref = unsafe {&plic::PLIC_REF};

    log!(Level::Info, "PLIC initialized");

    plic_ref.threshold_and_claim(current_context(), 0);
    plic_ref.set_priority(uart_int, 7);
    plic_ref.enable_int(current_context(), uart_int);

    uart.set_int();
    log::info!("UART interrupts set");

    hcf();
    
    syscon_rs::power_off()
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