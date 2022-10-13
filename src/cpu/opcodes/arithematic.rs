use crate::cpu::{ProgramCounter, Registers};
use crate::Cpu;
pub use crate::constants::*;
pub use crate::cpu::common::*;

/// This file contains all the functions necessary to support the Arithematic opcodes

impl Cpu {
    pub fn op_inx(&mut self, target: Registers) -> ProgramCounter {
        match target {
            Registers::SP | Registers::BC | Registers::DE | Registers::HL => {
                let mut pair: u16 = self.get_register_pair(target);
                pair = pair.overflowing_add(0x01).0;
                self.set_register_pair(target, pair);
            }
            _ => (),
        }
        ProgramCounter::Next
    }

    // DCX
    pub fn op_dcx(&mut self, reg: Registers) -> ProgramCounter {
        let mut val = self.get_register_pair(reg);
        val = val.overflowing_sub(1).0;
        self.set_register_pair(reg, val);

        ProgramCounter::Next
    }

    /// The specified byte is compared to the contents of the accumulator.
    /// The comparison is performed by internally subtracting the contents of REG from the accumulator
    /// (leaving both unchanged) and setting the condition bits according to the result.
    /// In particular, the Zero bit is set if the quantities are equal, and reset if they are unequal.
    /// Since a subtract operation is performed, the Carry bit will be set if there is no
    /// carry out of bit 7, indicating that the contents of REG are greater than the
    /// contents of the accumulator, and reset otherwise.
    pub fn op_cmp(&mut self, register: Registers) -> ProgramCounter {
        let min = self.a;
        let sub = match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };
        let res = min.overflowing_sub(sub).0;
        let ac = will_ac(min.wrapping_neg(), sub.wrapping_neg()); // Because it's a subtraction
        self.update_flags(res, Some(sub > min), Some(ac));

        ProgramCounter::Next
    }

    /// The specified byte is localled ``ORed`` bit by bit with the contents
    /// of the accumulator.  The carry bit is reset to zero.
    pub fn op_ora(&mut self, register: Registers) -> ProgramCounter {
        self.a |= match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);

        ProgramCounter::Next
    }

    /// The specified byte is locally ``XORed`` bit by bit with the contents
    /// of the accumulator.  The carry bit is reset to zero.
    pub fn op_xra(&mut self, register: Registers) -> ProgramCounter {
        let orig_value = self.a;
        let source_value = match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };
        let ac = will_ac(orig_value, source_value);
        self.a ^= source_value;

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, Some(ac));
        ProgramCounter::Next
    }

    /// The byte of immediate data is logically ```ANDed``` with the contents of the
    /// accumulator.  The carry bit is reset to zero.
    /// Bits affected: Carry, Zero, Sign, Parity
    pub fn op_ani(&mut self, dl: u8) -> ProgramCounter {
        self.a &= dl;
        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);

        ProgramCounter::Two
    }

    /// The specified byte is logically ``ANDed`` bit
    /// by bit with the contents of the accumulator. The Carry bit
    /// is reset to zero.
    pub fn op_ana(&mut self, register: Registers) -> ProgramCounter {
        self.a &= match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);
        ProgramCounter::Next
    }

    // Decimal Adjust Accumulator
    // If the least significant four bits of the accumulator have a value greater than nine,
    // or if the auxiliary carry flag is ON, DAA adds six to the accumulator.
    //
    // If the most significant four bits of the accumulator have a value greater than nine,
    // or if the carry flag IS ON, DAA adds six to the most significant four bits of the accumulator.
    pub fn op_daa(&mut self) -> ProgramCounter {
        // Find the LS4B of the accumulator
        let mut ac = false;
        let mut carry = false;

        if (self.a & 0b0000_1111) > 9 {
            let res = self.a.overflowing_add(6).0;
            ac = will_ac(6, self.a);
            self.a = res;
        }

        if (self.a & 0b1111_0000) > 9 {
            let (res, c) = self.a.overflowing_add(6 << 4);
            self.a = res;
            carry = c;
        }

        self.update_flags(self.a, Some(carry), Some(ac));

        ProgramCounter::Next
    }

    /// Add to the accumulator the supplied data byte after
    /// the opcode byte.
    ///
    /// If the parmeter ``carry_bit`` is true this performs the function
    /// of the opcode "ACI" by including the value of the carry bit in the
    /// addition.
    ///
    /// Condition bits affected: Carry, Sign, Zero, Parity, Aux Carry
    pub fn op_adi_aci(&mut self, dl: u8, carry_bit: bool) -> ProgramCounter {
        let mut to_add = dl;

        if carry_bit {
            to_add = to_add.overflowing_add(1).0; // Do we need to care about overflow here?
        };

        let ac = will_ac(to_add, self.a);
        let (res, of) = self.a.overflowing_add(to_add);
        self.a = res;
        self.update_flags(res, Some(of), Some(ac));

        ProgramCounter::Two
    }

    // Add to the accumulator the supplied register
    // along with the CARRY flag's value
    // as well as update flags
    pub fn op_adc(&mut self, register: Registers) -> ProgramCounter {
        let to_add: u8 = u8::from(self.test_flag(FLAG_CARRY))
            + match register {
                Registers::B => self.b,
                Registers::C => self.c,
                Registers::D => self.d,
                Registers::E => self.e,
                Registers::H => self.h,
                Registers::L => self.l,
                Registers::HL => self.memory[self.get_addr_pointer()],
                Registers::A => self.a,
                _ => 0_u8,
            };

        let (res, of) = self.a.overflowing_add(to_add);
        let ac = will_ac(to_add, self.a);
        self.a = res;
        self.update_flags(res, Some(of), Some(ac));

        ProgramCounter::Next
    }

    // Add to the accumulator the supplied register
    // as well as update flags
    pub fn op_add(&mut self, register: Registers) -> ProgramCounter {
        let to_add: u8 = match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };

        let (res, of) = self.a.overflowing_add(to_add);
        let ac = will_ac(to_add, self.a);
        self.a = res;
        self.update_flags(res, Some(of), Some(ac));

        ProgramCounter::Next
    }

    // Rotates right, if through_carry is true, it does that.
    pub fn op_rrc_rar(&mut self, through_carry: bool) -> ProgramCounter {
        // Store off our current carry bit
        let carry_bit = self.test_flag(FLAG_CARRY);
        let low_order = self.a & 0x01; // Save off the low order bit so we can rotate it.

        let mut new_accum: u8 = self.a >> 1;

        if through_carry {
            new_accum |= u8::from(carry_bit) << 7; // Carry bit replaces high order
        } else {
            // Normal carry
            new_accum |= low_order << 7; // Low order replaces high order
        };

        self.a = new_accum;

        if low_order > 0 {
            self.set_flag(FLAG_CARRY);
        } else {
            self.reset_flag(FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // CPI - Compare D16 to Accum, set flags accordingly
    pub fn op_cpi(&mut self, data: u8) -> ProgramCounter {
        // Subtract the data from register A and set flags on the result
        let (res, overflow) = self.a.overflowing_sub(data);
        let aux_carry = (self.a & 0x0F).wrapping_sub(data & 0x0F) > 0x0F;

        self.update_flags(res, Some(overflow), Some(aux_carry));

        ProgramCounter::Two
    }
}
