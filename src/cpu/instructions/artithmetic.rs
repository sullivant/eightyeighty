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
    pub fn op_cmp(&mut self, register: Registers) -> Result<(), String> {
        let min = self.a;
        let addr = self.get_addr_pointer();
        let mem_value = match self.memory().read(addr) {
            Ok(v) => v,
            Err(_) => {
                return Err("Invalid memory value at addr pointer".to_string());
            }
        };

        let sub = match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => mem_value,
            Registers::A => self.a,
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
        let mem_value = match self.memory().read(addr) {
            Ok(v) => v,
            Err(_) => {
                return Err("Invalid memory value at addr pointer".to_string());
            }
        };
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
                let val = mem_value;
                let ac = will_ac(1, val);
                let (res, of) = val.overflowing_add(1);
                self.update_flags(res, Some(of), Some(ac));
                self.memory().write(mem_value.into(), res).unwrap();
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
        let mem_value = match self.memory().read(addr) {
            Ok(v) => v,
            Err(_) => {
                return Err("Invalid memory value at addr pointer".to_string());
            }
        };

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
                let mem = mem_value;
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
    pub fn op_ora(&mut self, register: Registers) {
        let addr = self.get_addr_pointer();
        self.a |= match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory().read(addr).unwrap_or(0),
            Registers::A => self.a,
            _ => 0_u8,
        };

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, None);
    }

    /// The specified byte is locally ``XORed`` bit by bit with the contents
    /// of the accumulator.  The carry bit is reset to zero.
    pub fn op_xra(&mut self, register: Registers) {
        let orig_value = self.a;
        let addr = self.get_addr_pointer();
        let source_value = match register {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory().read(addr).unwrap_or(0),
            Registers::A => self.a,
            _ => 0_u8,
        };
        let ac = will_ac(orig_value, source_value);
        self.a ^= source_value;

        self.reset_flag(FLAG_CARRY);
        self.update_flags(self.a, None, Some(ac));
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO, OPCODE_SIZE};
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
}
