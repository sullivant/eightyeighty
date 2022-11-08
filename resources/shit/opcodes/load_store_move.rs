use crate::constants::FLAG_CARRY;
use crate::cpu::common::make_pointer;
use crate::cpu::{ProgramCounter, Registers};
use crate::Cpu;

/// This file contains the functions necessary to support the "Load / Store / Move" opcodes
///

impl Cpu {
    

    

    

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

    // Performs the MVI functionality
    pub fn op_mvi(&mut self, target: Registers, x: u8) -> ProgramCounter {
        match target {
            Registers::A => self.a = x,                                // 0x3E
            Registers::B => self.b = x,                                // 0x06
            Registers::C => self.c = x,                                // 0x0E
            Registers::D => self.d = x,                                // 0x16
            Registers::E => self.e = x,                                // 0x1E
            Registers::H => self.h = x,                                // 0x26
            Registers::L => self.l = x,                                // 0x2E
            Registers::HL => self.memory[self.get_addr_pointer()] = x, // 0x36
            _ => (),                                                   // Do nothing
        };
        ProgramCounter::Two
    }

    // Pops from the stack according to the register pair requested
    // 	L <- (sp); H <- (sp+1); sp <- sp+2
    pub fn op_pop(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                // BC Pair 0xC1
                self.c = self.memory[usize::from(self.sp)];
                self.b = self.memory[usize::from(self.sp + 1)];
            }
            Registers::D => {
                // DE Pair 0xD1
                self.e = self.memory[usize::from(self.sp)];
                self.d = self.memory[usize::from(self.sp + 1)];
            }
            Registers::H => {
                // HL Pair 0xE1
                self.l = self.memory[usize::from(self.sp)];
                self.h = self.memory[usize::from(self.sp + 1)];
            }
            Registers::SW => {
                // SW 0xF1
                self.flags = self.memory[usize::from(self.sp)];
                self.a = self.memory[usize::from(self.sp + 1)];
            }
            _ => (),
        };
        self.sp += 2;

        ProgramCounter::Next
    }

    // Loads the byte located at dhdl into the accumulator
    pub fn op_lda(&mut self, dl: u8, dh: u8) -> ProgramCounter {
        self.a = self.memory[make_pointer(dl, dh)];
        ProgramCounter::Three
    }

    // Exchanges registers DE with HL
    pub fn op_xchg(&mut self) -> ProgramCounter {
        let d = self.d;
        let e = self.e;

        self.d = self.h;
        self.e = self.l;
        self.h = d;
        self.l = e;

        ProgramCounter::Next
    }

    // Pushes onto stack according to the register pair requested
    // (sp-2)<-P2; (sp-1)<-P1; sp <- sp - 2
    pub fn op_push(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                // BC Pair 0xC5
                self.memory[usize::from(self.sp - 2)] = self.c;
                self.memory[usize::from(self.sp - 1)] = self.b;
            }
            Registers::D => {
                // DE Pair 0xD5
                self.memory[usize::from(self.sp - 2)] = self.e;
                self.memory[usize::from(self.sp - 1)] = self.d;
            }
            Registers::H => {
                // HL Pair 0xE5
                self.memory[usize::from(self.sp - 2)] = self.l;
                self.memory[usize::from(self.sp - 1)] = self.h;
            }
            Registers::SW => {
                // SW 0xF5
                self.memory[usize::from(self.sp - 2)] = self.flags;
                self.memory[usize::from(self.sp - 1)] = self.a;
            }
            _ => (),
        };
        self.sp -= 2;
        ProgramCounter::Next
    }
}
