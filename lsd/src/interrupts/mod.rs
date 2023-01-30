bitflags::bitflags! {
    pub struct Sie: u64 {
        const SEIE = 0b1000000000;    // Supervisor-level external interrupts
        const UEIE = 0b100000000;     // User-level external interrupts
        const STIE = 0b100000;        // Supervisor-level timer interrupts
        const UTIE = 0b10000;         // User-level timer interrupts
        const SSIE = 0b10;            // Supervisor-level software interrupts
        const USIE = 0b1;             // User-level software interrupts
    }

    pub struct Sstatus: u64 {
        const SIE = 0b10; // Supervisor-level interrupt enable
    }
}

impl Sie {
    /// Unsafe because it enables interrupts
    pub unsafe fn write_interrupts(&self) {
        core::arch::asm!(
            "csrw sie, {}",
            in(reg) self
        );
    }

    pub fn read_interrupts() -> u64 {
        let state: u64;

        unsafe {
            core::arch::asm!(
                "csrr {}, sie",
                out(reg) state
            );
        }

        state
    }
}

impl Sstatus {
    /// Unsafe because it enables interrupts
    pub unsafe fn write_interrupts(&self) {
        core::arch::asm!(
            "csrw sstatus, {}",
            in(reg) self
        );
    }

    pub fn read_interrupts() -> u64 {
        let state: u64;

        unsafe {
            core::arch::asm!(
                "csrr {}, sstatus",
                out(reg) state
            );
        }

        state
    }
}