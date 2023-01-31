static mut FREQUENCY: usize = 0;

pub fn init(fdt_ptr: *const u8) {
    unsafe {
        let fdt = fdt::Fdt::from_ptr(fdt_ptr).expect("Failed to get fdt");
        let cpu = fdt.cpus().next().unwrap();

        FREQUENCY = cpu.timebase_frequency();
    }
}

pub fn wait(time: Time) {
    sbi::legacy::set_timer(time.as_usize() as u64);
    super::wfi();
}

pub enum Time {
    Second(usize),
    Millisecond(usize),
    Microsecond(usize),
}

impl Time {
    pub fn as_usize(&self) -> usize {
        match self {
            Self::Second(val) => val * unsafe {FREQUENCY},
            Self::Millisecond(val) => val * unsafe {FREQUENCY}, 
            Self::Microsecond(val) => val * unsafe {FREQUENCY}, 
        }
    }
}