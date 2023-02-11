const PAGE_SIZE: usize = 4096;

pub struct Page([u8; PAGE_SIZE]);

#[derive(Debug)]
pub enum PagingType {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10
}

impl PagingType {
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "riscv,sv39" => Some(Self::Sv39),
            "riscv,sv48" => Some(Self::Sv48),
            "riscv,sv57" => Some(Self::Sv57),
            _ => None
        }
    }

    pub fn from_usize(id: usize) -> Self {
        match id {
            8 => Self::Sv39,
            9 => Self::Sv48,
            10 => Self::Sv57,
            _ => Self::Bare
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            Self::Sv39 => 8,
            Self::Sv48 => 9,
            Self::Sv57 => 10,
            _ => 0
        }
    }
}

bitflags::bitflags! {
    struct Entry: u64 {
        const V = 1 << 0; // Valid
        const R = 1 << 1; // Read
        const W = 1 << 2; // Write
        const X = 1 << 3; // Execute
        const U = 1 << 4; // User accessible
        const G = 1 << 5; // Global
        const A = 1 << 6; // Accessed
        const D = 1 << 7; // Dirty

        const RSW = 0b11 << 8; // Unknown

        const PPN0 = 0b111111111 << 10; // Physical Page Number 0
        const PPN1 = 0b111111111 << 19; // Physical Page Number 1
        const PPN2 = 0b111111111 << 28; // Physical Page Number 2
        const PPN3 = 0b111111111 << 37; // Physical Page Number 3
        const PPN4 = 0b11111111 << 46; // Physical Page Number 4

        const _RESERVED = 0b1111111 << 54; // Reserved

        const PBMT = 0b11 << 61; // Unknown

        const N = 0b1 << 63; // Unknown
    }
}

pub fn init() {}