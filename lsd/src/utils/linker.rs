extern {
    pub static KERNEL_START: LinkerSymbol;
    pub static KERNEL_END: LinkerSymbol;
    pub static __tdata_start: LinkerSymbol;
    pub static __tdata_end: LinkerSymbol;
    pub static __global_pointer: LinkerSymbol;
}

pub struct LinkerSymbol();

impl LinkerSymbol {
    pub fn as_ptr(&self) -> *const u8 {
        return self as *const Self as *const u8;
    }

    pub fn as_usize(&self) -> usize {
        return self.as_ptr() as usize;
    }
}