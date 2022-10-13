use crate::cpu::{ProgramCounter, Registers};
use crate::Cpu;
pub use crate::constants::*;

/// This file contains the functions necessary to support the "Load / Store / Move" opcodes
///

impl Cpu {
    // LXI (target pair), D16
    pub fn op_lxi(&mut self, target: Registers, x: u8, y: u8) -> ProgramCounter {
        match target {
            Registers::BC => {
                self.b = y;
                self.e = x;
            }
            Registers::DE => {
                self.d = y;
                self.e = x;
            }
            Registers::HL => {
                self.h = y;
                self.l = x;
            }
            Registers::SP => {
                self.sp = u16::from(y) << 8 | u16::from(x);
            }
            _ => (),
        }
        ProgramCounter::Three
    }

    // LHLD
    pub fn lhld(&mut self, dl: u8, dh: u8) -> ProgramCounter {
        let mut addr: u16 = u16::from(dh) << 8 | u16::from(dl);
        self.l = match self.memory.get(addr as usize) {
            Some(&v) => v,
            None => 0,
        };
        addr = addr.overflowing_add(0x01).0;
        self.h = match self.memory.get(addr as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Three
    }

    // MOV T(arget), Registers::X
    // Moves into T(arget) the value in register specified by the enum Registers
    pub fn op_mov(&mut self, target: Registers, source: Registers) -> ProgramCounter {
        let val = match source {
            Registers::A => self.a,
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::L => self.l,
            Registers::H => self.h,
            Registers::HL => self.memory[self.get_addr_pointer()],
            _ => {
                return ProgramCounter::Next;
            } // Ignored
        };

        match target {
            Registers::A => self.a = val,
            Registers::B => self.b = val,
            Registers::C => self.c = val,
            Registers::D => self.d = val,
            Registers::E => self.e = val,
            Registers::L => self.l = val,
            Registers::H => self.h = val,
            Registers::HL => self.memory[self.get_addr_pointer()] = val,
            _ => (), // Do nothing
        };

        ProgramCounter::Next
    }

    // Store accumulator direct to location in memory specified
    // by address dhdl
    pub fn op_sta(&mut self, dl: u8, dh: u8) -> ProgramCounter {
        let addr: u16 = u16::from(dh) << 8 | u16::from(dl);
        self.memory[addr as usize] = self.a;

        ProgramCounter::Three
    }


    // Stores accumulator at memory location of supplied register
    pub fn op_stax(&mut self, reg: Registers) -> ProgramCounter {
        // Get our location first
        let location = match reg {
            Registers::BC => Some(self.get_register_pair(Registers::BC)),
            Registers::DE => Some(self.get_register_pair(Registers::DE)),
            _ => None,
        };

        // Update memory with the value of the accumulator
        if let Some(l) = location {
            self.memory[l as usize] = self.a;
        }

        ProgramCounter::Next
    }

    // Rotates left, if through_carry is true, it does that.
    pub fn op_rlc_ral(&mut self, through_carry: bool) -> ProgramCounter {
        // Store off our current carry bit
        let carry_bit = self.test_flag(FLAG_CARRY);

        // Store off our current high order bit
        let high_order = self.a >> 7;

        // Shift one position to the left
        let mut new_accum: u8 = self.a << 1;

        if through_carry {
            new_accum |= u8::from(carry_bit); // carry bit replaces low order
        } else {
            new_accum |= high_order as u8; // high order replaces low order
        };

        self.a = new_accum;

        // Update the carry bit with the old high order bit
        if high_order > 0 {
            self.set_flag(FLAG_CARRY);
        } else {
            self.reset_flag(FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // LDAX
    // Loads A with value from memory location specified by register pair
    pub fn op_ldax(&mut self, reg: Registers) -> ProgramCounter {
        let loc: u16 = match reg {
            Registers::DE => u16::from(self.d) << 8 | u16::from(self.e),
            Registers::BC => u16::from(self.b) << 8 | u16::from(self.c),
            _ => {
                return ProgramCounter::Next;
            }
        };

        self.a = match self.memory.get(loc as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Next
    }
    
}
