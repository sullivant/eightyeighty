pub use crate::constants::*;
pub use crate::cpu::common::*;
use crate::{cpu::ProgramCounter, Cpu};

/// This file contains the functions needed for the "General / Control" opcodes
///

impl Cpu {
    // Similar to a jump, but it saves the current PC to the stack
    pub fn op_call(&mut self, x: u8, y: u8) -> ProgramCounter {
        // Save away the current PC hi/lo into the stack
        let pc_hi = self.pc >> 8;
        let pc_lo = self.pc & 0xFF;

        self.memory[usize::from(self.sp - 1)] = pc_hi as u8;
        self.memory[usize::from(self.sp - 2)] = pc_lo as u8;
        self.sp -= 2;

        // Tell the program counter where we want to go next
        let ys: u16 = u16::from(y) << 8;
        self.pc = usize::from(ys | u16::from(x));

        ProgramCounter::Jump(self.pc)
    }

    // Calls if the flag's supplied value matches the supplied sign
    pub fn op_call_if(&mut self, flag: u8, sign: bool, x: u8, y: u8) -> ProgramCounter {
        if sign == self.test_flag(flag) {
            return self.op_call(x, y);
        }
        ProgramCounter::Three
    }

    // Jumps if the flag's supplied value matches the supplied sign
    pub fn op_jump_if(&mut self, flag: u8, sign: bool, x: u8, y: u8) -> ProgramCounter {
        if sign == self.test_flag(flag) {
            self.pc = make_pointer(x, y);

            ProgramCounter::Jump(self.pc)
        } else {
            ProgramCounter::Three
        }
    }

    // Returns if the flag supplied's value matches the supplied sign
    pub fn op_rets(&mut self, flag: u8, sign: bool) -> ProgramCounter {
        if sign == self.test_flag(flag) {
            return self.op_ret();
        }

        ProgramCounter::Next
    }

    // RET (PC.lo <- (sp); PC.hi<-(sp+1); SP <- SP+2)
    pub fn op_ret(&mut self) -> ProgramCounter {
        let pc_lo = match self.memory.get(usize::from(self.sp)) {
            Some(&v) => v,
            None => 0,
        };
        let pc_hi = match self.memory.get(usize::from(self.sp + 1)) {
            Some(&v) => v,
            None => 0,
        };
        let dest: u16 = u16::from(pc_hi) << 8 | u16::from(pc_lo);
        self.pc = dest as usize; // Set our PC back to where we were
        self.sp += 2;

        //ProgramCounter::Jump(dest.into())
        ProgramCounter::Three // And go to the next op
    }

    // Jump to a given location as provided by (y<<8 | x)
    #[must_use]
    pub fn op_jmp(x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        ProgramCounter::Jump(dest.into())
    }

    // JZ (Jump if zero)
    pub fn op_jz(&mut self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        if self.test_flag(FLAG_ZERO) {
            ProgramCounter::Jump(dest.into())
        } else {
            ProgramCounter::Three
        }
    }

    // JNZ (Jump if nonzero)
    pub fn op_jnz(&mut self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        if self.test_flag(FLAG_ZERO) {
            ProgramCounter::Three
        } else {
            ProgramCounter::Jump(dest.into())
        }
    }

    // JNC (Jump if No Carry)
    pub fn op_jnc(&mut self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);

        if self.test_flag(FLAG_CARRY) {
            ProgramCounter::Three
        } else {
            ProgramCounter::Jump(dest.into())
        }
    }

    // The contents of the program counter (16bit)
    // are pushed onto the stack, providing a return address for
    // later use by a RETURN instruction.
    // Program execution continues at memory address:
    // OOOOOOOOOOEXPOOOB
    pub fn op_rst(&mut self, loc: u8) -> ProgramCounter {
        self.memory[usize::from(self.sp - 2)] = (self.pc as u16 >> 8) as u8;
        self.memory[usize::from(self.sp - 1)] = (self.pc as u16 & 0xFF) as u8;
        self.sp -= 2;

        ProgramCounter::Jump((loc << 3) as usize)
    }
}