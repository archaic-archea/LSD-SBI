pub struct Bitmap {
    buffer: *mut u8,
    size: usize
}

impl Bitmap {
    pub const fn new(buffer: *const u8, size: usize) -> Self {
        Bitmap { buffer: buffer.cast_mut(), size }
    }

    pub fn read(&self, index: usize) -> bool {
        let byte_addr = index >> 3;
        let bit_addr = index & 0b111;

        unsafe {
            let byte =  self.buffer.add(byte_addr).read_volatile();

            return ((byte >> bit_addr) & 0b1) != 0;
        }
    }

    pub fn set(&mut self, index: usize) {
        let byte_addr = index >> 3;
        let bit_addr = index & 0b111;

        unsafe {
            let read = self.buffer.add(byte_addr).read_volatile();

            self.buffer.write_volatile(read | (1 << bit_addr));
        }
    }

    pub fn clear(&self, index: usize) {
        let byte_addr = index >> 3;
        let bit_addr = index & 0b111;

        unsafe {
            let read = self.buffer.add(byte_addr).read_volatile();

            self.buffer.write_volatile(read & (!(1 << bit_addr)));
        }
    }
}