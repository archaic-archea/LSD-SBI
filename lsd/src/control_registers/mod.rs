bitflags::bitflags! {
    pub struct Sie: u64 {
        const SEIE = 1 << 9;    // Supervisor-level external interrupts
        const UEIE = 1 << 8;     // User-level external interrupts
        const STIE = 1 << 5;        // Supervisor-level timer interrupts
        const UTIE = 1 << 4;         // User-level timer interrupts
        const SSIE = 1 << 1;                // Supervisor-level software interrupts
        const USIE = 1 << 0;                // User-level software interrupts
    }

    pub struct Sstatus: u64 {
        const SIE = 1 << 1; // Supervisor-level interrupt enable
    }
}

impl Sie {
    pub fn supervisor_all() -> Self {
        Self::SEIE | Self::SSIE | Self::STIE
    }
    /// Unsafe because it enables interrupts
    pub unsafe fn write(&self) {
        core::arch::asm!(
            "csrw sie, {}",
            in(reg) self.bits()
        );
    }

    pub fn read() -> Self {
        let state: u64;

        unsafe {
            core::arch::asm!(
                "csrr {}, sie",
                out(reg) state
            );
        }

        unsafe {Self::from_bits_unchecked(state)}
    }
}

impl Sstatus {
    /// Unsafe because it enables interrupts
    pub unsafe fn write(&self) {
        core::arch::asm!(
            "csrw sstatus, {}",
            in(reg) self.bits()
        );
    }

    pub fn read() -> Self {
        let state: u64;

        unsafe {
            core::arch::asm!(
                "csrr {}, sstatus",
                out(reg) state
            );
        }

        unsafe {Self::from_bits_unchecked(state)}
    }
}