/// This is the magic that translates things for stuff like the Space Invaders hardware; 
/// video is stored in one orientation in RAM and we need to shift it out into a pattern
/// that will match raster scanning orientation on a CRT.

#[derive(Debug, Default, Clone)]
pub struct ShiftRegister {
    register: u16,
    shift_offset: u8,
}

impl ShiftRegister {
    pub fn new() -> Self {
        Self {
            register: 0,
            shift_offset: 0,
        }
    }

    // Write to the low portion of the register
    pub fn write_low(&mut self, value: u8) {
        self.register = (self.register & 0xFF00) | (value as u16);
    }

    // Write to the high portion of the register
    pub fn write_high(&mut self, value: u8) {
        self.register = (self.register & 0x00FF) | ((value as u16) << 8);
    }

    // Set the offset and only really care about 0-7
    pub fn set_offset(&mut self, offset: u8) {
        self.shift_offset = offset & 0x07;
    }

    pub fn read_shifted(&self) -> u8 {
        let shift = 8 - self.shift_offset;
        ((self.register >> shift) & 0xFF) as u8
    }
}