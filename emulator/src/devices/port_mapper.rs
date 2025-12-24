use crate::{bus::IoDevice, devices::{InputLatch, ShiftRegister}};

/// This really becomes a "Space Invaders" specific port mapping, and in the future I'd like to migrate this to be
/// chosen by the user at startup.  Flags "--space-invaders" or "--midway" or "--cpm" or something in a ui.  I am trying 
/// to imagine it being similar to the way a spring java setup might utilize a "factory" to pick the right configuration
/// and device here.

pub struct PortMapper {
    pub input_latch0: InputLatch,
    pub input_latch1: InputLatch,
    pub input_latch2: InputLatch,
    pub shift_register: ShiftRegister
}

impl IoDevice for PortMapper {
    fn input(&mut self, port: u8) -> u8 {
        match port {
            0 => self.input_latch0.read(),
            1 => self.input_latch1.read(),
            2 => self.input_latch2.read(),
            3 => self.shift_register.read_shifted(),
            _ => 0,
        }
    }

    fn output(&mut self, port: u8, value: u8) {
        match port {
            2 => self.shift_register.set_offset(value),
            4 => self.shift_register.write_low(value),
            5 => self.shift_register.write_high(value),
            _ => (),
        }
    }
}