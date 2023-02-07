extern "C" {
    pub static KERNEL_START: LinkerSymbol;
    pub static KERNEL_END: LinkerSymbol;
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