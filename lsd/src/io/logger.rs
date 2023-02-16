use core::fmt::Write;

use spin::Mutex;

static UART: Mutex<Uart> = Mutex::new(Uart::new(0x1000_0000));

struct Uart(u64);

impl Uart {
    const fn new(base_addr: u64) -> Uart {
        Uart(base_addr)
    }

    fn write(&self, string: &str) {
        let ptr = self.0 as *mut u8;

        for c in string.chars() {
            unsafe {
                ptr.write_volatile(c as u8);
            }
        }
    }
}


impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);

        Ok(())
    }
}

pub fn log_fmt(args: core::fmt::Arguments) {
    let mut uart = UART.lock();
    uart.write_fmt(args).expect("Failed to write to UART");
}

#[macro_export]
macro_rules! log_print {
    ($($t:tt)*) => { $crate::io::logger::log_fmt(format_args!($($t)*)) };
}

#[macro_export]
macro_rules! log_println {
    () => { $crate::print!("\n") };
    ($($t:tt)*) => { $crate::io::logger::log_fmt(format_args!("{}\n", format_args!($($t)*))) };
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        log_println!("{}: {}", record.target(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    //log::log!(log::Level::Debug, "Logger initialized");
}