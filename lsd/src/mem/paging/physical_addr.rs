use super::virtual_addr::VirtualAddress;

#[derive(Debug)]
pub struct PhyscialAddress(u64);

impl PhyscialAddress {
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn to_virt(&self) -> VirtualAddress {
        VirtualAddress::new(self.0)
    }

    pub fn sections(&self) -> PhysSections {
        let addr = self.0;

        let page_offset = addr & 0b1111_1111_1111;
        let ppn0 = (addr >> 12) & 0b1_1111_1111;
        let ppn1 = (addr >> 21) & 0b1_1111_1111;
        let ppn2 = (addr >> 30) & 0b1_1111_1111;
        let ppn3 = (addr >> 39) & 0b1_1111_1111;
        let ppn4 = (addr >> 48) & 0b1111_1111;

        PhysSections {
            page_offset,
            ppn0,
            ppn1,
            ppn2,
            ppn3,
            ppn4
        }
    }
}

pub struct PhysSections {
    page_offset: u64,
    ppn0: u64,
    ppn1: u64,
    ppn2: u64,
    ppn3: u64,
    ppn4: u64,
}

pub struct Ppn(u64);

impl Ppn {
    pub fn from_phys(addr: PhyscialAddress) -> Self {
        let address = addr.0;

        Self(address >> 12)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}