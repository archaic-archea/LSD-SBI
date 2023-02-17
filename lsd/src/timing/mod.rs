use core::sync::atomic;
use core::sync::atomic::Ordering;

static FREQUENCY: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
pub static WAIT: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn init(fdt_ptr: *const u8) {
    unsafe {
        let fdt = fdt::Fdt::from_ptr(fdt_ptr).expect("Failed to get fdt");
        let cpu = fdt.cpus().next().unwrap();

        FREQUENCY.store(cpu.timebase_frequency(), Ordering::Relaxed);
    }
}

pub fn wait(time: Time) {
    sbi::legacy::set_timer(time.as_usize() as u64);

    WAIT.store(true, Ordering::Relaxed);
    while WAIT.load(Ordering::Relaxed) {
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
            Self::Hour(val) => val * FREQUENCY.load(Ordering::Relaxed) * 3600,
            Self::Minute(val) => val * FREQUENCY.load(Ordering::Relaxed) * 60,
            Self::Second(val) => val * FREQUENCY.load(Ordering::Relaxed),
            Self::Millisecond(val) => (val / 1000) * FREQUENCY.load(Ordering::Relaxed), 
            Self::Microsecond(val) => (val / 1000000) * FREQUENCY.load(Ordering::Relaxed), 
        }
    }
}