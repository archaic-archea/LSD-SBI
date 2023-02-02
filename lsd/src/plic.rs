pub struct PlicRefer(*mut Plic);

impl PlicRefer {
    pub fn new(ptr: *const u8) -> Self {
        Self(ptr.cast_mut() as *mut Plic)
    }

    pub fn init(&self, max_interrupts: usize, context: usize) {
        for i in 1..max_interrupts {
            self.set_priority(i, 0);
        }

        for i in 0..max_interrupts {
            self.disable_int(context, i);
        }

        self.threshold_and_claim(context, 0);
    }

    pub fn set_priority(&self, index: usize, priority: u32) {
        unsafe {
            (*self.0).source_priorities[index] = priority;
        }
    }

    pub fn threshold_and_claim(&self, context: usize, thresh: u32) {
        unsafe {
            (*self.0).threshold_and_claim[context][0] = thresh;
        }
    }

    pub fn enable_int(&self, context: usize, index: usize) {
        let addr = (index >> 5) & 0b11111;
        let bit_addr = index & 0b11111;

        unsafe {
            (*self.0).interrupt_enable[context][addr] |= 1 << bit_addr;
        }
    }

    pub fn disable_int(&self, context: usize, index: usize) {
        let addr = (index >> 5) & 0b11111;
        let bit_addr = index & 0b11111;

        unsafe {
            (*self.0).interrupt_enable[context][addr] &= !(1 << bit_addr);
        }
    }
}

#[repr(C)]
pub struct Plic {
    pub(crate) source_priorities: [u32; 1024], // 4096 bytes = 0x1000
    pub(crate) interrupt_pending: [u32; 32], // 128 bytes = 0x80
    _padding1: [u8; 3968], // 3968 bytes = 0xF80
    pub(crate) interrupt_enable: [[u32; 32]; 15872], // 2031616 bytes = 0x1F0000
    _padding2: [u8; 57344], // 57344 bytes = 0xE000
    pub(crate) threshold_and_claim: [[u32; 1024]; 15872], // 65011712 bytes = 0x3E00000
    _padding3: [u8; 4088] // 4088 bytes
} // 4096 + 128 + 3968 + 2031616 + 57344 + 65011712 = 67108864 = 0x400_0000

impl super::Compat for Plic {
    fn compatible() -> &'static [&'static str] {
        &["riscv,plic0"]
    }
}