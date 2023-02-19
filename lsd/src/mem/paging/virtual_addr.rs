use super::physical_addr::PhyscialAddress;

#[derive(Debug, Clone, Copy)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn to_phys(&self) -> PhyscialAddress {
        PhyscialAddress::new(self.0)
    }

    pub fn to_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub fn sections(&self) -> VirtSections {
        let addr = self.0;

        let page_offset = addr & 0b1111_1111_1111;
        let vpn0 = (addr >> 12) & 0b1_1111_1111;
        let vpn1 = (addr >> 21) & 0b1_1111_1111;
        let vpn2 = (addr >> 30) & 0b1_1111_1111;
        let vpn3 = (addr >> 39) & 0b1_1111_1111;
        let vpn4 = (addr >> 48) & 0b1_1111_1111;

        VirtSections {
            page_offset,
            vpn0,
            vpn1,
            vpn2,
            vpn3,
            vpn4
        }
    }
}

#[derive(Debug)]
pub struct VirtSections {
    pub page_offset: u64,
    pub vpn0: u64,
    pub vpn1: u64,
    pub vpn2: u64,
    pub vpn3: u64,
    pub vpn4: u64,
}

impl VirtSections {
    pub fn address(&self) -> VirtualAddress {
        let mut address = self.page_offset;
        address += self.vpn0 << 12;
        address += self.vpn1 << 21;
        address += self.vpn2 << 30;
        address += self.vpn3 << 39;
        address += self.vpn4 << 48;

        VirtualAddress(address)
    }
}

impl core::ops::Index<usize> for VirtSections {
    type Output = u64;

    fn index(&self, idx: usize) -> &u64 {
        match idx {
            0 => &self.vpn4,
            1 => &self.vpn3,
            2 => &self.vpn2,
            3 => &self.vpn1,
            4 => &self.vpn0,
            _ => &self.page_offset
        }
    }
}