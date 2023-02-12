bitflags::bitflags! {
    pub struct Sie: usize {
        const SEIE = 1 << 9;    // Supervisor-level external interrupts
        const UEIE = 1 << 8;    // User-level external interrupts
        const STIE = 1 << 5;    // Supervisor-level timer interrupts
        const UTIE = 1 << 4;    // User-level timer interrupts
        const SSIE = 1 << 1;    // Supervisor-level software interrupts
        const USIE = 1 << 0;    // User-level software interrupts
    }

    pub struct Sstatus: usize {
        const SIE = 1 << 1; // Supervisor-level interrupt enable
    }

    pub struct Satp: usize {
        const MODE_MASK = 0xf << 60;

        const ASID_MASK = 0xffff << 44;

        const PPN_MASK = 0xf_ffff_ffff << 0;
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
        let state: usize;

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
        let state: usize;

        unsafe {
            core::arch::asm!(
                "csrr {}, sstatus",
                out(reg) state
            );
        }

        unsafe {Self::from_bits_unchecked(state)}
    }
}

impl Satp {
    pub fn read_mode() -> crate::mem::paging::PagingType {
        let read = Self::read() & !Self::MODE_MASK.bits();

        crate::mem::paging::PagingType::from_usize(read >> 60)
    }

    pub fn write_mode(paging_mode: crate::mem::paging::PagingType) {
        let read = Self::read() & !Self::MODE_MASK.bits();

        let write = read | paging_mode.as_usize();

        Self::write(write);
    }

    pub fn write_state(satp: SatpState) {
        let mut state: usize;

        state = satp.ppn as usize;
        state += (satp.asid as usize) << 44;
        state += satp.mode.as_usize() << 60;

        Self::write(state);
    }
    
    pub fn read_state() -> SatpState {
        let mut state = SatpState {
            mode: crate::mem::paging::PagingType::Bare,
            asid: 0,
            ppn: 0
        };

        let read = Self::read();

        let mode = Self::read_mode();
        let asid = (read >> 44) & Self::ASID_MASK.bits();
        let ppn = (read >> 0) & Self::PPN_MASK.bits();

        state.mode = mode;
        state.asid = asid as u16;
        state.ppn = ppn as u64;

        state
    }

    pub fn write(data: usize) {
        unsafe {
            core::arch::asm!(
                "csrw satp, {}",
                in(reg) data
            )
        }
    }

    pub fn read() -> usize {
        let state: usize;

        unsafe {
            core::arch::asm!(
                "csrr {}, satp",
                out(reg) state
            );
        }

        state
    }
}

pub struct SatpState {
    mode: crate::mem::paging::PagingType,
    asid: u16,
    ppn: u64
}

use crate::mem::paging::physical_addr::Ppn;

impl SatpState {
    pub fn new(mode: crate::mem::paging::PagingType, asid: u16, ppn: Ppn) -> Self {
        Self { 
            mode,
            asid,
            ppn: ppn.as_u64()
        }
    }
}