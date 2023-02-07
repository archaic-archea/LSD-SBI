use core::ptr::addr_of;

pub static mut PLIC_REF: PlicRefer = PlicRefer(core::ptr::null_mut());

pub fn init(devicetree_ptr: *const u8, context: usize) {
    use crate::Compat;

    let fdt: fdt::Fdt;
    unsafe {
        fdt = fdt::Fdt::from_ptr(devicetree_ptr).unwrap();
    }

    let plic_node = fdt.find_compatible(Plic::compatible()).expect("Failed to find plic");
    let plic_region = plic_node.reg().expect("No plic region").next().unwrap();
    
    PlicRefer::init_plic_ref(plic_region.starting_address);
    let plic_ref = unsafe {&PLIC_REF};

    plic_ref.init(11, context);
}

pub struct PlicRefer(*mut Plic);

impl PlicRefer {
    pub fn init_plic_ref(ptr: *const u8) {
        let plic_ref = Self(ptr.cast_mut() as *mut Plic);
        unsafe {PLIC_REF = plic_ref};
    }

    pub fn init(&self, max_interrupts: usize, context: usize) {
        for i in 1..max_interrupts {
            self.set_priority(i, 0);
        }
        log::info!("Test"); // Idk why but if I remove this it breaks

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

    pub fn enable_int(&self, context: usize, intr: usize) {
        // Sanity checks, neither values would be valid
        if context >= 15872 || intr >= 1024 {
            return;
        }

        let (index, bit) = (intr / 32, intr % 32);
    
        log::info!(
            "[context={context}] Enabling interrupt 0x{intr:x} @ {:#p} [index={index}, bit={bit}]",
            unsafe { addr_of!((*self.0).interrupt_enable[context][index]) }
        );
    
        unsafe {
            (*self.0).interrupt_enable[context][index] |= 1 << bit;
        }
    }
    
    pub fn disable_int(&self, context: usize, intr: usize) {
        // Sanity checks, neither values would be valid
        if context >= 15872 || intr >= 1024 {
            return;
        }
        
        let (index, bit) = (intr / 32, intr % 32);
    
        log::info!(
            "[context={context}] Disabling interrupt 0x{intr:x} @ {:#p} [index={index}, bit={bit}]",
            unsafe { addr_of!((*self.0).interrupt_enable[context][index]) }
        );
    
        unsafe {
            (*self.0).interrupt_enable[context][index] &= !(1 << bit);
        }
    }

    pub fn pending(&self) -> u32 {
        unsafe {
            (*self.0).interrupt_pending[0]
        }
    }

    pub fn next(&self, context: usize) -> Option<u32> {
        let id = unsafe {(*self.0).threshold_and_claim[context][1]};

        if id == 0 {
            None
        } else {
            Some(id)
        }
    }

    pub fn claim(&self, context: usize, id: u32) {
        // Sanity checks, neither values would be valid
        if context >= 15872 || id >= 1024 {
            return;
        }

        let threshold_and_claim = unsafe {&mut (*self.0).threshold_and_claim};

        threshold_and_claim[context][1] = id;
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