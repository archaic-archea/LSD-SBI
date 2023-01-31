use log::{log, Level};

pub static mut FDT: *const u8 = core::ptr::null();

pub fn init(fdt: *const u8) {
    unsafe {
        FDT = fdt;
    }
}

pub fn power_off() ->  Option<()> {
    let fdt = unsafe {fdt::Fdt::from_ptr(FDT).unwrap()};
    
    let poweroff_node = fdt.find_compatible(&["syscon-poweroff"])?;
    let offset = poweroff_node.property("offset")?.as_usize()?;
    let value = poweroff_node.property("value")?.as_usize()? as u32;
    let syscon_phandle = poweroff_node.property("regmap")?.as_usize()? as u32;
    let syscon_node = fdt.find_phandle(syscon_phandle)?;
    let syscon_mmio = syscon_node.reg()?.next()?.starting_address.cast::<u32>().cast_mut();
    log!(Level::Info, "Powering off");
    unsafe {
        syscon_mmio.byte_add(offset).write_volatile(value);
    }

    Some(())
}

pub fn reboot() ->  Option<()> {
    let fdt = unsafe {fdt::Fdt::from_ptr(FDT).unwrap()};
    
    let reboot = fdt.find_compatible(&["syscon-reboot"])?;
    let offset = reboot.property("offset")?.as_usize()?;
    let value = reboot.property("value")?.as_usize()? as u32;
    let syscon_phandle = reboot.property("regmap")?.as_usize()? as u32;
    let syscon_node = fdt.find_phandle(syscon_phandle)?;
    let syscon_mmio = syscon_node.reg()?.next()?.starting_address.cast::<u32>().cast_mut();
    log!(Level::Info, "Rebooting...");
    unsafe {
        syscon_mmio.byte_add(offset).write_volatile(value);
    }

    Some(())
}