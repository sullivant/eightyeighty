/// Allows for a latched input mechanism

pub struct InputLatch {
    value: u8,
}

impl InputLatch {
    #[must_use]
    pub fn new() -> Self {
        Self { value: 0 }
    }

    /// Reads the current latched value but does not remove it
    #[inline]
    #[must_use]
    pub fn read(&self) -> u8 {
        self.value
    }

    /// Sets a single bit (0-7)
    #[inline]
    pub fn set_bit(&mut self, bit: u8) {
        if bit < 8 {
            self.value |= 1 << bit;  // set bit to 1
        }
    }


    /// Clears a single bit (0-7)
    #[inline]
    pub fn clear_bit(&mut self, bit: u8) {
        if bit < 8 {
            self.value &= !(1 << bit);  // clear bit to 0
        }
    }


    /// Sets or clears depending on value
    #[inline]
    pub fn write_bit(&mut self, bit: u8, state: bool) {
        if state {
            self.set_bit(bit);
        } else {
            self.clear_bit(bit);
        }
    }

    /// Overwrites all bits at once
    #[inline]
    pub fn write(&mut self, value: u8) {
        self.value = value;
    }
}