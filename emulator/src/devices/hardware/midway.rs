/// This is the hardware configuration available for a Midway 8080 (Space Invaders) system

use crate::devices::io::{InputLatch, ShiftRegister};
use crate::bus::IoDevice;


/// Inputs that a Midway expects
#[derive(Debug)]
pub enum MidwayInput {
    Coin,
    Start1,
    Start2,
    Fire,
    Left,
    Right,
    Tilt
}

pub struct MidwayHardware {
    pub input_latch0: InputLatch,
    pub input_latch1: InputLatch,
    pub input_latch2: InputLatch,
    pub shift_register: ShiftRegister
}

impl IoDevice for MidwayHardware {
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
    fn set_port(&mut self, port: u8, value: u8) {
        match port {
            0 => self.input_latch0.write(value),
            1 => self.input_latch1.write(value),
            2 => self.input_latch2.write(value),
            _ => {}
        }
    }

    fn set_bit(&mut self, port: u8, bit: u8) {
        // ACTIVE LOW
        match port {
            0 => self.input_latch0.write_bit(bit, false),
            1 => self.input_latch1.write_bit(bit, false),
            2 => self.input_latch2.write_bit(bit, false),
            _ => {}
        }
    }

    fn clear_bit(&mut self, port: u8, bit: u8) {
        match port {
            0 => self.input_latch0.write_bit(bit, true),
            1 => self.input_latch1.write_bit(bit, true),
            2 => self.input_latch2.write_bit(bit, true),
            _ => {}
        }
    }
}

impl MidwayHardware {
    pub fn new() -> Self {
        let mut in0 = InputLatch::new();
        let mut in1 = InputLatch::new();
        let mut in2 = InputLatch::new();

        // Set them all to be pulled high - they start out as "off" but are ACTIVE LOW
        in0.write(0xFF);
        in1.write(0xFF);
        in2.write(0xFF);

        // Default DIP switches
        in2.set_bit(0); // 3 lives
        in2.set_bit(1);

        let shift = ShiftRegister::new();

        Self {
            input_latch0: in0,
            input_latch1: in1,
            input_latch2: in2,
            shift_register: shift
        }
    }

    /// Assert or clear a logical input
    pub fn set_input(&mut self, input: MidwayInput, pressed: bool) {
        let level = !pressed; // Midway is ACTIVE LOW

        match input {
            // IN0
            MidwayInput::Coin => self.input_latch0.write_bit(0, level),
            MidwayInput::Tilt => self.input_latch0.write_bit(2, level),

            // IN1 (player controls)
            MidwayInput::Start1 => self.input_latch1.write_bit(2, level),
            MidwayInput::Start2 => self.input_latch1.write_bit(1, level),
            MidwayInput::Fire => self.input_latch1.write_bit(4, level),
            MidwayInput::Left => self.input_latch1.write_bit(5, level),
            MidwayInput::Right => self.input_latch1.write_bit(6, level),
        }
    }

    /// Convenience helpers
    pub fn press(&mut self, input: MidwayInput) {
        self.set_input(input, true);
    }

    pub fn release(&mut self, input: MidwayInput) {
        self.set_input(input, false);
    }
}




