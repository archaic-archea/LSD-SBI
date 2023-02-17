use sifive_plic::*;

pub static mut PLIC_REF: *mut Plic = core::ptr::null_mut();

pub fn init(devicetree_ptr: *const u8, contexts: impl Iterator<Item = usize>) {
    log::info!("PLIC initializing...");

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).unwrap();
    }

    let plic_node = fdt.find_compatible(Plic::compatible()).expect("Failed to find plic");
    let plic_region = plic_node.reg().expect("No plic region").next().unwrap();
    
    unsafe {
        PLIC_REF = plic_region.starting_address.cast_mut() as *mut Plic;
    }
    let plic_ref = unsafe {&mut *PLIC_REF};

    plic_ref.init(11, contexts);

    log::info!("PLIC Enabled")
}