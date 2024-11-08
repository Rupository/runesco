pub struct AddrRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0), // high byte first, lo byte second (big endian!)
            hi_ptr: true, // a flag to check whether the next byte (u8) will be writter at hi or lo
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8; // hi
        self.value.1 = (data & 0xff) as u8; // lo
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            // if next write should be at hi
            self.value.0 = data;
        } else {
            self.value.1 = data; // next write at lo
        }

        if self.get() > 0x3fff {
            //mirror down addr above 0x3fff
            self.set(self.get() & 0b11111111111111);
        }

        self.hi_ptr = !self.hi_ptr; // next write should be opp of prev write (hi -> lo, lo -> hi)
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc); // increase by the required amount (1 or 32)

        if lo > self.value.1 {
            // if the inc caused an overflow (otherwas we should have lo < self.value.1)
            self.value.0 = self.value.0.wrapping_add(1); // increment hi
        }

        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111); //mirror down addr above 0x3fff
        }
    }

    pub fn reset_latch(&mut self) {
        // reset
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        // get full address
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}
