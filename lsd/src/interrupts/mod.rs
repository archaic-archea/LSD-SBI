pub enum InterruptTypes {
    SEIE,               // Unknown
    UEIE,               // Unknown
    STIE = 0b100000,    // Supervisor-level timer interrupt
    UTIE = 0b10000,     // User-level timer interrupts
    SSIE,               // Unknown
    USIE                // Unknown
}

impl InterruptTypes {
    pub fn as_u16(&self) -> u16 {
        match self {
            Self::STIE => Self::STIE as u16,
            Self::UTIE => Self::UTIE as u16,
            _ => panic!("Interrupt type not implemented")
        }
    }

    /// Unsafe because it can enable certain interrupts
    /// Sets the bit enabling an interrupt type
    pub unsafe fn set_int_type(&self) {
        let bit = self.as_u16();

        let current: u16;

        core::arch::asm!(
            "csrr {}, sie",
            out(reg) current
        );

        let res = current | bit;

        core::arch::asm!(
            "csrw sie, {}",
            in(reg) res
        );
    }

    /// Unsafe because it can disable certain interrupts
    /// Clears the bit enabling an interrupt type
    pub unsafe fn clear_int_type(&self) {
        let bit_mask = !self.as_u16();

        let current: u16;

        core::arch::asm!(
            "csrr {}, sie",
            out(reg) current
        );

        let res = current & bit_mask;

        core::arch::asm!(
            "csrw sie, {}",
            in(reg) res
        );
    }
}