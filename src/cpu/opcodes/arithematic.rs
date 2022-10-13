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

    // Performs the Double Add (DAD) functionality
    // Sets H to the value according to the supplied register
    // Basically: HL = HL+<Selected register pair>
    pub fn op_dad(&mut self, source: Registers) -> ProgramCounter {
        //let val = usize::from(u16::from(self.h) << 8 | u16::from(self.l));
        let val = usize::from(self.get_register_pair(Registers::HL));

        let src: usize = match source {
            Registers::B | Registers::BC => usize::from(self.get_register_pair(Registers::BC)),
            Registers::D | Registers::DE => usize::from(self.get_register_pair(Registers::DE)),
            Registers::SP => usize::from(self.get_register_pair(Registers::SP)),
            Registers::H | Registers::HL => val,
            _ => 0,
        };

        let (new, of) = val.overflowing_add(src);

        self.h = (new >> 8) as u8;
        self.l = (new & 0xFF) as u8;

        if of {
            self.set_flag(FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // DCR Reg
    // Flags affected: Z,S,P,AC
    #[allow(clippy::similar_names)]
    pub fn op_dcr(&mut self, reg: Registers) -> ProgramCounter {
        //let new_val = self.b.wrapping_sub(1);

        match reg {
            Registers::A => {
                let (res, of) = self.b.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.a & 0x0F)));
                self.a = res;
            }
            Registers::B => {
                let (res, of) = self.b.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.b & 0x0F)));
                self.b = res;
            }
            Registers::C => {
                let (res, of) = self.c.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.c & 0x0F)));
                self.c = res;
            }
            Registers::D => {
                let (res, of) = self.d.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.d & 0x0F)));
                self.d = res;
            }
            Registers::E => {
                let (res, of) = self.e.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.e & 0x0F)));
                self.e = res;
            }
            Registers::H => {
                let (res, of) = self.h.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.h & 0x0F)));
                self.h = res;
            }
            Registers::L => {
                let (res, of) = self.l.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (self.l & 0x0F)));
                self.l = res;
            }
            Registers::HL => {
                let mem = self.memory[self.get_addr_pointer()];
                let (res, of) = mem.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (mem & 0x0F)));
                self.memory[self.get_addr_pointer()] = res;
            }

            _ => (),
        }

        ProgramCounter::Next
    }

    // SUB  / SBB (Subtract register param from A with borrow if necessary)
    // Additionally, an optional subtrahend can be supplied, in the case of SBB
    // and it will be included in the subtraction
    //
    // Flags affected: Z, S, P, CY, AC
    pub fn op_sub(&mut self, reg: Registers, sub: u8) -> ProgramCounter {
        let o: (u8, bool) = match reg {
            Registers::A => self.a.overflowing_sub(self.a.overflowing_add(sub).0),
            Registers::B => self.a.overflowing_sub(self.b.overflowing_add(sub).0),
            Registers::C => self.a.overflowing_sub(self.c.overflowing_add(sub).0),
            Registers::D => self.a.overflowing_sub(self.d.overflowing_add(sub).0),
            Registers::E => self.a.overflowing_sub(self.e.overflowing_add(sub).0),
            Registers::H => self.a.overflowing_sub(self.h.overflowing_add(sub).0),
            Registers::L => self.a.overflowing_sub(self.l.overflowing_add(sub).0),
            Registers::HL => self
                .a
                .overflowing_sub(self.memory[self.get_addr_pointer()].overflowing_add(sub).0),
            _ => (0_u8, false),
        };

        let ac = will_ac(o.0.wrapping_neg(), self.a.wrapping_neg()); // Because it's a subtraction

        //self.update_flags(o.0, o.1, (1 & 0x0F) > (self.a & 0x0F));
        self.update_flags(o.0, Some(o.1), Some(ac));
        self.a = o.0;
        ProgramCounter::Next
    }

    // INR Reg
    // Flags affected: Z,S,P,AC
    #[allow(clippy::similar_names)]
    pub fn op_inr(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                let (res, of) = self.b.overflowing_add(1);
                let ac = will_ac(1, self.b);
                self.update_flags(res, Some(of), Some(ac));
                self.b = res;
            }
            Registers::C => {
                let (res, of) = self.c.overflowing_add(1);
                let ac = will_ac(1, self.c);
                self.update_flags(res, Some(of), Some(ac));
                self.c = res;
            }
            Registers::D => {
                let (res, of) = self.d.overflowing_add(1);
                let ac = will_ac(1, self.d);
                self.update_flags(res, Some(of), Some(ac));
                self.d = res;
            }
            Registers::E => {
                let (res, of) = self.e.overflowing_add(1);
                let ac = will_ac(1, self.d);
                self.update_flags(res, Some(of), Some(ac));
                self.e = res;
            }
            Registers::H => {
                let (res, of) = self.h.overflowing_add(1);
                let ac = will_ac(1, self.h);
                self.update_flags(res, Some(of), Some(ac));
                self.h = res;
            }
            Registers::L => {
                let (res, of) = self.l.overflowing_add(1);
                let ac = will_ac(1, self.l);
                self.update_flags(res, Some(of), Some(ac));
                self.l = res;
            }
            Registers::HL => {
                let val = self.memory[self.get_addr_pointer()];
                let ac = will_ac(1, val);
                let (res, of) = val.overflowing_add(1);
                self.update_flags(res, Some(of), Some(ac));
                self.memory[self.get_addr_pointer()] = res;
            }
            Registers::A => {
                let (res, of) = self.a.overflowing_add(1);
                let ac = will_ac(1, self.a);
                self.update_flags(res, Some(of), Some(ac));
                self.a = res;
            }
            _ => (),
        }

        ProgramCounter::Next
    }

    // Sets a register to the compliment of itself
    pub fn op_comp(&mut self, register: Registers) -> ProgramCounter {
        if let Registers::A = register {
            self.a = !self.a;
        }
        ProgramCounter::Next
    }

    // Sets the carry flag to the compliment of itself
    pub fn op_cmc(&mut self) -> ProgramCounter {
        if self.test_flag(FLAG_CARRY) {
            // Flag needs to be reset
            self.reset_flag(FLAG_CARRY);
        } else {
            // Flag needs to be set
            self.set_flag(FLAG_CARRY);
        }
        ProgramCounter::Next
    }

    // Sets the carry flag
    pub fn op_stc(&mut self) -> ProgramCounter {
        self.set_flag(FLAG_CARRY);
        ProgramCounter::Next
    }
}
