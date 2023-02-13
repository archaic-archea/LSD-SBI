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
    mem::init(devicetree_ptr);
    interrupts::init();
    HART_ID.store(hartid, core::sync::atomic::Ordering::Relaxed);
    plic::init(devicetree_ptr, current_context()..current_context() + 1);
    mem::paging::init();

    log::info!("Pagining initialized");

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).expect("Failed to get fdt");
    }

    let uart_node = fdt.find_compatible(uart::Uart16550::compatible()).expect("Failed to find Uart");
    let uart_int = uart_node.property("interrupts").unwrap().as_usize().unwrap();
    let uart_reg = uart_node.reg().unwrap().next().unwrap();
    let uart = unsafe {&*(uart_reg.starting_address.cast_mut() as *mut uart::Uart16550)};

    uart.init();
    uart.set_int();

    let plic_ref = unsafe {&mut *plic::PLIC_REF};

    plic_ref.set_context_threshold(current_context(), 0);
    plic_ref.set_interrupt_priority(uart_int, 7);
    plic_ref.enable_interrupt(current_context(), uart_int);

    let mut paging_type = mem::paging::PagingType::Sv39;

    for node in fdt.all_nodes() {
        if node.name.contains("cpu@") {
            log::info!("Node: {} {{", node.name);
            for prop in node.properties() {
                log::info!("    Cpu property: {}", prop.name);
                log::info!("        {}: {:?}", prop.name, prop.as_str());
            }
            log::info!("}}");

            let mmu_type = node.property("mmu-type").expect("No MMu type");
            paging_type = mem::paging::PagingType::from_str(mmu_type.as_str().unwrap()).expect("Invalid MMU type");
        } else {
            log::info!("Node: {}", node.name);
        }
    }

    log::info!("Paging type: {:?}\n", paging_type);

    let memmap_free = unsafe {&mem::MEMMAP.free};
    let free_limit = memmap_free.max();
    log::info!("Free base:   {:#?}", memmap_free.base());
    log::info!("Free limit:  {:#?}", free_limit);
    log::info!("Free length: 0x{:x}\n", memmap_free.length());

    let mem = unsafe {&mem::MEMMAP.mem};
    let limit = mem.max();
    log::info!("Mem base:    {:#?}", mem.base());
    log::info!("Mem limit:   {:#?}", limit);
    log::info!("Mem length:  0x{:x}", mem.length());

    hcf();
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
