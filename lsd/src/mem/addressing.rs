pub struct PhyscialAddress(usize);


pub struct VirtualAddress(usize);


pub struct Ppn(usize);

impl Ppn {
    pub fn from_phys(addr: PhyscialAddress) -> Self {
        let address = addr.0;

        Self(address >> 12)
    }
}