use crate::{
    constants::FLAG_CARRY,
    cpu::{will_ac, Registers, CPU},
};

impl CPU {
    pub fn op_inx(&mut self, target: Registers) {
        match target {
            Registers::SP | Registers::BC | Registers::DE | Registers::HL => {
                let mut pair: u16 = self.get_register_pair(target);
                pair = pair.overflowing_add(0x01).0;
                self.set_register_pair(target, pair);
            }
            _ => (),
        }
    }

    // DCX
    pub fn op_dcx(&mut self, reg: Registers) {
        let mut val = self.get_register_pair(reg);
        val = val.overflowing_sub(1).0;
        self.set_register_pair(reg, val);
    }

    /// The specified byte is compared to the contents of the accumulator.
    /// The comparison is performed by internally subtracting the contents of REG from the accumulator
    /// (leaving both unchanged) and setting the condition bits according to the result.
    /// In particular, the Zero bit is set if the quantities are equal, and reset if they are unequal.
    /// Since a subtract operation is performed, the Carry bit will be set if there is no
    /// carry out of bit 7, indicating that the contents of REG are greater than the
    /// contents of the accumulator, and reset otherwise.
    pub fn op_cmp(&mut self) -> Result<(), String> {
        let min = self.a;
        let addr = self.get_addr_pointer();

        let Ok(value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

        let sub = match self.current_instruction.opcode {
            0xB8 => self.b,
            0xB9 => self.c,
            0xBA => self.d,
            0xBB => self.e,
            0xBC => self.h,
            0xBD => self.l,
            0xBE => value,
            0xBF => self.a,
            _ => 0_u8,
        };
        let res = min.overflowing_sub(sub).0;
        let ac = will_ac(min.wrapping_neg(), sub.wrapping_neg()); // Because it's a subtraction
        self.update_flags(res, Some(sub > min), Some(ac));

        Ok(())
    }

    // INR Reg
    // Flags affected: Z,S,P,AC
    #[allow(clippy::similar_names)]
    pub fn op_inr(&mut self, reg: Registers) -> Result<(), String> {
        let addr = self.get_addr_pointer();
        let Ok(value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

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
                let val = value;
                let ac = will_ac(1, val);
                let (res, of) = val.overflowing_add(1);
                self.update_flags(res, Some(of), Some(ac));
                self.memory().write(value.into(), res).unwrap();
            }
            Registers::A => {
                let (res, of) = self.a.overflowing_add(1);
                let ac = will_ac(1, self.a);
                self.update_flags(res, Some(of), Some(ac));
                self.a = res;
            }
            _ => (),
        }
        Ok(())
    }

    // DCR Reg
    // Flags affected: Z,S,P,AC
    #[allow(clippy::similar_names)]
    pub fn op_dcr(&mut self, reg: Registers) -> Result<(), String> {
        let addr = self.get_addr_pointer();
        let Ok(value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

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
                let mem = value;
                let (res, of) = mem.overflowing_sub(1);
                self.update_flags(res, Some(of), Some((1 & 0x0F) > (mem & 0x0F)));
                match self.memory().write(addr, res) {
                    Ok(_) => (),
                    Err(_) => {
                        return Err("Unable to write to memory value at addr pointer".to_string());
                    }
                }
            }

            _ => (),
        }

        Ok(())
    }

    /// The specified byte is localled ``ORed`` bit by bit with the contents
    /// of the accumulator.  The carry bit is reset to zero.
    pub fn op_ora(&mut self) -> Result<(), String> {
        let opcode = self.current_instruction.opcode;
        let addr = self.get_addr_pointer();
        let Ok(mem_value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

        self.a |= match opcode {
            0xB0 => self.b,
            0xB1 => self.c,
            0xB2 => self.d,
            0xB3 => self.e,
            0xB4 => self.h,
            0xB5 => self.l,
            0xB6 => mem_value,
            0xB7 => self.a,
            _ => 0_u8,
        };

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);

        Ok(())
    }

    /// The specified byte is logically ``ANDed`` bit
    /// by bit with the contents of the accumulator. The Carry bit
    /// is reset to zero.
    pub fn op_ana(&mut self) -> Result<(), String> {
        let addr = self.get_addr_pointer();
        let Ok(mem_value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

        self.a &= match self.current_instruction.opcode {
            0xA0 => self.b,
            0xA1 => self.c,
            0xA2 => self.d,
            0xA3 => self.e,
            0xA4 => self.h,
            0xA5 => self.l,
            0xA6 => mem_value,
            0xA7 => self.a,
            _ => 0_u8,
        };

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);
        Ok(())
    }

    /// The byte of immediate data is logically ```ANDed``` with the contents of the
    /// accumulator.  The carry bit is reset to zero.
    /// Bits affected: Carry, Zero, Sign, Parity
    pub fn op_ani(&mut self, dl: u8) {
        self.a &= dl;
        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);
    }

    /// The specified byte is locally ``XORed`` bit by bit with the contents
    /// of the accumulator.  The carry bit is reset to zero.
    pub fn op_xra(&mut self) -> Result<(), String> {
        let orig_value = self.a;
        let addr = self.get_addr_pointer();
        let Ok(mem_value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

        let source_value = match self.current_instruction.opcode {
            0xA8 => self.b,
            0xA9 => self.c,
            0xAA => self.d,
            0xAB => self.e,
            0xAC => self.h,
            0xAD => self.l,
            0xAE => mem_value,
            0xAF => self.a,
            _ => 0_u8,
        };
        let ac = will_ac(orig_value, source_value);
        self.a ^= source_value;

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, Some(ac));

        Ok(())
    }

    /// SUB  / SBB (Subtract register param from A with borrow if necessary)
    /// Additionally, an optional subtrahend can be supplied, in the case of SBB
    /// and it will be included in the subtraction
    ///
    /// This function will use the current instruction (opcode) to determine which
    /// register to use.
    ///
    /// Flags affected: Z, S, P, CY, AC
    pub fn op_sub(&mut self) -> Result<(), String> {
        let opcode = self.current_instruction.opcode;
        let sub = self.get_flag(FLAG_CARRY);

        let addr = self.get_addr_pointer();
        let Ok(mem_value) = self.memory().read(addr) else { return Err("Invalid memory value at addr pointer".to_string()); };

        let o: (u8, bool) = match opcode {
            0x90 => self.a.overflowing_sub(self.b.overflowing_add(0).0),
            0x91 => self.a.overflowing_sub(self.c.overflowing_add(0).0),
            0x92 => self.a.overflowing_sub(self.d.overflowing_add(0).0),
            0x93 => self.a.overflowing_sub(self.e.overflowing_add(0).0),
            0x94 => self.a.overflowing_sub(self.h.overflowing_add(0).0),
            0x95 => self.a.overflowing_sub(self.l.overflowing_add(0).0),
            0x96 => self.a.overflowing_sub(mem_value.overflowing_add(0).0),
            0x97 => self.a.overflowing_sub(self.a.overflowing_add(0).0),
            0x98 => self.a.overflowing_sub(self.b.overflowing_add(sub).0),
            0x99 => self.a.overflowing_sub(self.c.overflowing_add(sub).0),
            0x9A => self.a.overflowing_sub(self.d.overflowing_add(sub).0),
            0x9B => self.a.overflowing_sub(self.e.overflowing_add(sub).0),
            0x9C => self.a.overflowing_sub(self.h.overflowing_add(sub).0),
            0x9D => self.a.overflowing_sub(self.l.overflowing_add(sub).0),
            0x9E => self.a.overflowing_sub(mem_value.overflowing_add(sub).0),
            0x9F => self.a.overflowing_sub(self.a.overflowing_add(sub).0),
            _ => (0_u8, false),
        };

        let ac = will_ac(o.0.wrapping_neg(), self.a.wrapping_neg()); // Because it's a subtraction

        //self.update_flags(o.0, o.1, (1 & 0x0F) > (self.a & 0x0F));
        self.update_flags(o.0, Some(o.1), Some(ac));
        self.a = o.0;
        Ok(())
    }

    /// Decimal Adjust Accumulator
    /// If the least significant four bits of the accumulator have a value greater than nine,
    /// or if the auxiliary carry flag is ON, DAA adds six to the accumulator.
    ///
    /// If the most significant four bits of the accumulator have a value greater than nine,
    /// or if the carry flag IS ON, DAA adds six to the most significant four bits of the accumulator.
    pub fn op_daa(&mut self) {
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
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{
        FLAG_AUXCARRY, FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO, OPCODE_SIZE,
    };
    use crate::cpu::CPU;

    #[test]
    fn test_op_inx() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.b = 0x18;
        cpu.c = 0xff;
        cpu.prep_instr_and_data(0x03, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.pc, (op + OPCODE_SIZE));
        assert_eq!(cpu.b, 0x19);
        assert_eq!(cpu.c, 0x00);

        // try again with the overflow protection
        cpu.b = 0xff;
        cpu.c = 0xff;
        cpu.prep_instr_and_data(0x03, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.c, 0x00);
    }

    #[test]
    fn test_dcx() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.d = 0x20;
        cpu.e = 0x00;
        cpu.sp = 0x1234;

        cpu.prep_instr_and_data(0x1B, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.d, 0x1F);
        assert_eq!(cpu.e, 0xFF);
        assert_eq!(cpu.pc, op + (OPCODE_SIZE));

        cpu.prep_instr_and_data(0x3B, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.sp, 0x1233);
    }

    #[test]
    fn test_op_cmp() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.a = 0x0A;
        cpu.e = 0x05;
        cpu.set_flag(FLAG_CARRY);

        cpu.prep_instr_and_data(0xBB, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x0A);
        assert_eq!(cpu.e, 0x05);
        assert_eq!(cpu.test_flag(FLAG_CARRY), false);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);

        cpu.a = 0x02;
        cpu.prep_instr_and_data(0xBB, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x02);
        assert_eq!(cpu.e, 0x05);
        assert_eq!(cpu.test_flag(FLAG_CARRY), true);

        cpu.a = !0x1B;
        cpu.prep_instr_and_data(0xBB, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, !0x1B);
        assert_eq!(cpu.e, 0x05);
        assert_eq!(cpu.test_flag(FLAG_CARRY), false);
    }

    #[test]
    fn test_op_inr() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.e = 0x99;
        cpu.prep_instr_and_data(0x1C, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.e, 0x9A);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
    }

    #[test]
    fn test_op_dcr() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // A simple decrement
        cpu.b = 0x02;
        cpu.prep_instr_and_data(0x05, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.b, 0x01);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.test_flag(FLAG_ZERO), false);
        cpu.prep_instr_and_data(0x05, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.test_flag(FLAG_ZERO), true);

        // A wrapping decrement
        cpu.b = 0x00;
        cpu.prep_instr_and_data(0x05, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.b, 0xFF);
        assert_eq!(cpu.test_flag(FLAG_SIGN), true);

        cpu.b = 0x04;
        cpu.prep_instr_and_data(0x05, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.b, 0x03);
        assert_eq!(cpu.test_flag(FLAG_PARITY), true);
    }

    #[test]
    fn test_op_xra() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.a = 0xFC;

        // Should zero out the A register
        cpu.prep_instr_and_data(0xAF, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);

        cpu.a = 0xFF;
        cpu.b = 0b0000_1010;
        cpu.prep_instr_and_data(0xA8, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0b1111_0101);
    }

    #[test]
    fn test_op_ora() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.a = 0x33;
        cpu.c = 0x0F;
        cpu.set_flag(FLAG_CARRY);

        // Should zero out the A register
        cpu.prep_instr_and_data(0xB1, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x3F);
        assert_eq!(cpu.test_flag(FLAG_CARRY), false);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
    }

    #[test]
    fn test_op_ana() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.a = 0xFC;
        cpu.c = 0x0F;

        cpu.prep_instr_and_data(0xA1, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
    }

    #[test]
    fn test_op_ani() {
        let mut cpu = CPU::new();

        // Setup the accumulator with 0x3A
        cpu.prep_instr_and_data(0x3E, 0x3A, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x3A);
        let op = cpu.pc;

        // Try ANI with 0xFF for the data
        cpu.prep_instr_and_data(0xE6, 0x0F, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x0A);

        assert_eq!(cpu.pc, op + OPCODE_SIZE * 2);
    }

    #[test]
    fn test_sub() {
        let mut cpu = CPU::new();
        let op = cpu.pc;
        cpu.a = 0x12;
        cpu.c = 0x02;

        cpu.prep_instr_and_data(0x91, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + (OPCODE_SIZE));
        assert_eq!(cpu.a, 0x10);

        cpu.a = 0x3E;
        cpu.prep_instr_and_data(0x97, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.test_flag(FLAG_PARITY), true);
        assert_eq!(cpu.test_flag(FLAG_ZERO), true);

        cpu.memory().write(0x2400, 0x01).unwrap();
        cpu.h = 0x24;
        cpu.l = 0x00;
        cpu.a = 0x0B;
        cpu.prep_instr_and_data(0x96, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x0A);
    }

    #[test]
    fn test_op_daa() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Setup the accum with 0x9B and reset both carry bits
        cpu.a = 0x9b;
        cpu.reset_flag(FLAG_AUXCARRY);
        cpu.reset_flag(FLAG_CARRY);

        cpu.prep_instr_and_data(0x27, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.a, 0x01);
        assert!(cpu.test_flag(FLAG_CARRY));
        assert!(cpu.test_flag(FLAG_AUXCARRY));
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
    }
}
