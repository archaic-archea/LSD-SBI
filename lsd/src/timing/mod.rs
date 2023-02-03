static mut FREQUENCY: usize = 0;
pub static mut WAIT: bool = false;

pub fn init(fdt_ptr: *const u8) {
    unsafe {
        let fdt = fdt::Fdt::from_ptr(fdt_ptr).expect("Failed to get fdt");
        let cpu = fdt.cpus().next().unwrap();

        FREQUENCY = cpu.timebase_frequency();
    }
}

pub fn wait(time: Time) {
    sbi::legacy::set_timer(time.as_usize() as u64);

    unsafe {WAIT = true;}
    while unsafe {WAIT} {
        super::wfi();
    }
}

pub enum Time {
    Hour(usize),
    Minute(usize),
    Second(usize),
    Millisecond(usize),
    Microsecond(usize),
}

impl Time {
    pub fn as_usize(&self) -> usize {
        match self {
            Self::Hour(val) => val * unsafe {FREQUENCY} * 3600,
            Self::Minute(val) => val * unsafe {FREQUENCY} * 60,
            Self::Second(val) => val * unsafe {FREQUENCY},
            Self::Millisecond(val) => (val / 1000) * unsafe {FREQUENCY}, 
            Self::Microsecond(val) => (val / 1000000) * unsafe {FREQUENCY}, 
        }
    }
}