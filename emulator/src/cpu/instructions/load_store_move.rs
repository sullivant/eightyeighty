use crate::{
    constants::FLAG_CARRY,
    cpu::{make_pointer, Registers, CPU},
};

/// This contains any instructions of the LOAD / STORE / MOVE category
impl CPU {
    /// The registers HL replace the contents of the SP
    pub fn sphl(&mut self) {
        self.sp = make_pointer(self.l, self.h);
    }

    /// Contents of L are exchanged with contents of memory byte whose
    /// address is held in the stack pointer SP.  The contents of H are
    /// exchanged with the contents of the memory byte whose address is
    /// one greater than that held in the stack pointer SP.
    pub fn xthl(&mut self) -> Result<(), String> {
        // Store away our temp values
        let ch = self.h;
        let cl = self.l;

        // Pop the sp into the new values
        match self.pop(Registers::HL) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        // Push the "old" ones
        match self.push(cl, ch) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    /// Exchanges the contents of the H and L registers with the contents of the
    /// D and E registers.
    pub fn xchg(&mut self) {
        let oh = self.h;
        let ol = self.l;

        self.h = self.d;
        self.l = self.e;

        self.d = oh;
        self.e = ol;
    }

    /// Pushes onto the stack the values provided to this function.  They are,
    /// most likely and often, the values contained in a register pair such as `BC`
    ///
    /// They are pushed on like this:
    /// (sp-1)<-dh; (sp-2)<-dl; sp <- sp - 2
    pub fn push(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        self.sp -= 1;
        match self.memory.write(self.sp.into(), dh) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "PUSH: Unable to write to SP location {0}, error is: {e}",
                    self.sp
                ))
            }
        }

        self.sp -= 1;
        match self.memory.write(self.sp.into(), dl) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "PUSH: Unable to write to SP location {0}, error is: {e}",
                    self.sp
                ))
            }
        }

        Ok(())
    }

    /// Pops from the stack according to the register pair requested
    /// L <- (sp); H <- (sp+1); sp <- sp+2
    pub fn pop(&mut self, reg: Registers) -> Result<(), String> {
        // Gather our two values we're popping
        let source_a = match self.memory.read(self.sp.into()) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "POP: Unable to read from SP location {0}, error is: {e}",
                    self.sp
                ))
            }
        };
        let source_b = match self.memory.read((self.sp + 1).into()) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "POP: Unable to read from SP location {0}, error is: {e}",
                    self.sp + 1
                ))
            }
        };

        match reg {
            Registers::BC => {
                self.c = source_a;
                self.b = source_b;
            }
            Registers::DE => {
                self.e = source_a;
                self.d = source_b;
            }
            Registers::HL => {
                self.l = source_a;
                self.h = source_b;
            }
            Registers::SW => {
                self.flags = source_a;
                self.a = source_b;
            }
            _ => return Err(format!("POP: Invalid source register requested: {reg}")),
        };

        self.sp += 2;

        Ok(())
    }

    /// Stores a copy of the L register in the memory location specified in bytes
    /// two and three of this instruction and then stores a copy of the H register
    /// in the next higher memory location.
    pub fn shld(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let addr: u16 = make_pointer(dl, dh);

        match self.memory.write(addr as usize, self.l) {
            Ok(_v) => (),
            Err(e) => {
                return Err(format!(
                    "SHLD: Unable to write L to memory at {addr:#04X}, error is: {e}"
                ))
            }
        };
        match self.memory.write((addr + 1) as usize, self.h) {
            Ok(_v) => (),
            Err(e) => {
                return Err(format!(
                    "SHLD: Unable to write H to memory at {addr:#04X}, error is: {e}"
                ))
            }
        }

        Ok(())
    }

    /// Rotates accumulator left (RLC), if `through_carry` is true, it
    /// will roate accumulator left, through the carry bit (RAL), too.
    pub fn rlc_ral(&mut self, through_carry: bool) {
        // Store off our current carry bit
        let carry_bit = self.test_flag(FLAG_CARRY);

        // Store off our current accumulator's high order bit
        let high_order = self.a >> 7;

        // Rotate accum left
        let mut new_accum: u8 = self.a << 1;

        if through_carry {
            // RAR
            // Set carry bit to high order
            self.update_flag(FLAG_CARRY, high_order != 0);

            // Set low order to prior carry bit
            new_accum |= u8::from(carry_bit);
        } else {
            // RLC
            // Set carry bit to high order
            self.update_flag(FLAG_CARRY, high_order != 0);

            // High order bit transfers to low order bit
            new_accum |= high_order;
        }

        self.a = new_accum;
    }

    /// LDA
    /// Loads the accumulator with a copy of the byte at the location specified
    /// in bytes 2 and 3 of the instruction
    pub fn lda(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let addr: u16 = make_pointer(dl, dh);
        self.a = match self.memory.read(addr as usize) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "LHLD: Unable to read memory at {addr:#04X}, error is {e}"
                ))
            }
        };

        Ok(())
    }

    /// LDAX
    /// Loads the accumulator with the contents of the memory location indicated by
    /// the register pair (B or D).
    pub fn ldax(&mut self, target: Registers) -> Result<(), String> {
        let addr: u16 = match target {
            Registers::BC => self.get_register_pair(Registers::BC),
            Registers::DE => self.get_register_pair(Registers::DE),
            _ => {
                return Err(format!(
                    "LDAX: Invalid register pair for LDAX instruction: {target}"
                ))
            }
        };

        self.a = match self.memory.read(addr as usize) {
            Ok(v) => v,
            Err(_) => return Err(format!("LDAX: Unable to read memory at {addr:#04X}")),
        };

        Ok(())
    }

    /// LXI (target pair), D16
    /// Loads into the target pair the source data (dl and dh)
    pub fn lxi(&mut self, target: Registers, dl: u8, dh: u8) -> Result<(), String> {
        match target {
            Registers::BC => {
                self.b = dh;
                self.c = dl;
                Ok(())
            }
            Registers::DE => {
                self.d = dh;
                self.e = dl;
                Ok(())
            }
            Registers::HL => {
                self.h = dh;
                self.l = dl;
                Ok(())
            }
            Registers::SP => {
                self.sp = make_pointer(dl, dh);
                Ok(())
            }
            _ => Err(format!(
                "Register {target} is NOT IMPLEMENTED in OP_LXI, Cannot Execute"
            )),
        }
    }

    // LHLD - loads into HL pair the values in the location at the supplied address
    pub fn lhld(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let mut addr: u16 = u16::from(dh) << 8 | u16::from(dl);
        self.l = match self.memory.read(addr as usize) {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "LHLD: Unable to read for L in memory at {addr:#04X}"
                ))
            }
        };
        addr = addr.overflowing_add(0x01).0;
        self.h = match self.memory.read(addr as usize) {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "LHLD: Unable to read for H in memory at {addr:#04X}"
                ))
            }
        };

        Ok(())
    }

    // MOV T(arget), Registers::X
    // Moves into T(arget) the value in register specified by the enum Registers
    pub fn mov(&mut self, target: Registers, source: Registers) -> Result<(), String> {
        let addr = self.get_addr_pointer();
        let val = match source {
            Registers::A => self.a,
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::L => self.l,
            Registers::H => self.h,
            Registers::HL => match self.memory.read(addr) {
                Ok(v) => v,
                Err(e) => return Err(e),
            },
            _ => {
                return Err(format!("Cannot MOV from unimplemented register: {source}"));
            }
        };

        match target {
            Registers::A => self.a = val,
            Registers::B => self.b = val,
            Registers::C => self.c = val,
            Registers::D => self.d = val,
            Registers::E => self.e = val,
            Registers::L => self.l = val,
            Registers::H => self.h = val,
            Registers::HL => match self.memory.write(addr, val) {
                Ok(()) => (),
                Err(e) => return Err(e),
            },
            _ => {
                return Err(format!("Cannot MOV into unimplemented register: {source}"));
            }
        };

        Ok(())
    }

    // Store accumulator direct to location in memory specified
    // by address dhdl
    pub fn sta(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let addr: usize = usize::from(u16::from(dh) << 8 | u16::from(dl));
        self.memory.write(addr, self.a)
    }

    // Stores accumulator at memory location of supplied register
    pub fn stax(&mut self, reg: Registers) -> Result<(), String> {
        // Get our location first
        let location = match reg {
            Registers::BC => Some(self.get_register_pair(Registers::BC)),
            Registers::DE => Some(self.get_register_pair(Registers::DE)),
            _ => None,
        };

        // Update memory with the value of the accumulator
        if let Some(l) = location {
            return self.memory.write(l as usize, self.a);
        }

        Err(format!(
            "Cannot determine location from register pair provided {reg:#}"
        ))
    }

    // Performs the MVI functionality
    pub fn mvi(&mut self, x: u8) -> Result<(), String> {
        let addr = self.get_addr_pointer();

        match self.current_instruction.opcode {
            0x06 => self.b = x,                    // 0x06
            0x0E => self.c = x,                    // 0x0E
            0x16 => self.d = x,                    // 0x16
            0x1E => self.e = x,                    // 0x1E
            0x26 => self.h = x,                    // 0x26
            0x2E => self.l = x,                    // 0x2E
            0x36 => self.memory().write(addr, x)?, // 0x36
            0x3E => self.a = x,                    // 0x3E
            _ => (),                               // Do nothing
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{
        FLAG_AUXCARRY, FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO, OPCODE_SIZE,
    };
    use crate::cpu::{Registers, CPU};

    #[test]
    fn test_sphl() {
        let mut cpu = CPU::new();
        cpu.h = 0x50;
        cpu.l = 0x6C;

        cpu.prep_instr_and_data(0xF9, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.sp, 0x506C);
    }

    #[test]
    fn test_xthl() {
        let mut cpu = CPU::new();
        cpu.sp = 0x10AD;
        cpu.h = 0x0B;
        cpu.l = 0x3C;
        cpu.memory.write(0x10AD, 0xF0).unwrap();
        cpu.memory.write(0x10AE, 0x0D).unwrap();

        cpu.prep_instr_and_data(0xE3, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.h, 0x0D);
        assert_eq!(cpu.l, 0xF0);
        assert_eq!(cpu.memory.read(0x10AD).unwrap(), 0x3C);
        assert_eq!(cpu.memory.read(0x10AE).unwrap(), 0x0B);
    }

    #[test]
    fn test_push() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.b = 0x8F;
        cpu.c = 0x9D;
        cpu.sp = 0x3A2C;
        cpu.prep_instr_and_data(0xC5, 0x00, 0x00); // PUSH B
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.memory.read(0x3A2B).unwrap(), 0x8F);
        assert_eq!(cpu.memory.read(0x3A2A).unwrap(), 0x9D);
        assert_eq!(cpu.sp, 0x3A2A);

        cpu.d = 0x8F;
        cpu.e = 0x9D;
        cpu.sp = 0x3B2C;
        cpu.memory.write(0x3B2B, 0x00).unwrap();
        cpu.memory.write(0x3B2A, 0x00).unwrap();
        cpu.prep_instr_and_data(0xD5, 0x00, 0x00); // PUSH D
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.memory.read(0x3B2B).unwrap(), 0x8F);
        assert_eq!(cpu.memory.read(0x3B2A).unwrap(), 0x9D);
        assert_eq!(cpu.sp, 0x3B2A);

        cpu.h = 0x8F;
        cpu.l = 0x9D;
        cpu.sp = 0x3F2C;
        cpu.memory.write(0x3F2B, 0x00).unwrap();
        cpu.memory.write(0x3F2A, 0x00).unwrap();
        cpu.prep_instr_and_data(0xE5, 0x00, 0x00); // PUSH H
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.memory.read(0x3F2B).unwrap(), 0x8F);
        assert_eq!(cpu.memory.read(0x3F2A).unwrap(), 0x9D);
        assert_eq!(cpu.sp, 0x3F2A);

        cpu.a = 0x1F;
        cpu.sp = 0x502A;
        cpu.set_flag(FLAG_CARRY);
        cpu.set_flag(FLAG_ZERO);
        cpu.set_flag(FLAG_PARITY);
        cpu.reset_flag(FLAG_SIGN);
        cpu.reset_flag(FLAG_AUXCARRY);
        cpu.prep_instr_and_data(0xF5, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.memory.read(0x5029).unwrap(), 0x1F);
        assert_eq!(cpu.memory.read(0x5028).unwrap(), 0x47); // The PSW setup with the flags, above
        assert_eq!(cpu.sp, 0x5028);
    }

    #[test]
    fn test_pop() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.memory().write(0x1239, 0x3D).unwrap();
        cpu.memory().write(0x123A, 0x93).unwrap();
        cpu.sp = 0x1239;
        cpu.prep_instr_and_data(0xC1, 0x00, 0x00); // POP B
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.c, 0x3D);
        assert_eq!(cpu.b, 0x93);
        assert_eq!(cpu.sp, 0x123B);

        cpu.sp = 0x1239;
        cpu.prep_instr_and_data(0xD1, 0x00, 0x00); // POP D
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.e, 0x3D);
        assert_eq!(cpu.d, 0x93);
        assert_eq!(cpu.sp, 0x123B);

        cpu.sp = 0x1239;
        cpu.prep_instr_and_data(0xE1, 0x00, 0x00); // POP H
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.c, 0x3D);
        assert_eq!(cpu.b, 0x93);
        assert_eq!(cpu.sp, 0x123B);

        cpu.memory().write(0x2C00, 0xC3).unwrap();
        cpu.memory().write(0x2C01, 0xFF).unwrap();
        cpu.sp = 0x2C00;
        cpu.prep_instr_and_data(0xF1, 0x00, 0x00); // POP PSW
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.get_flags(), 0xC3); // PSW is 11000011 (0xC3)
                                           // Check the flags individually anyway..
        assert!(cpu.test_flag(FLAG_SIGN));
        assert!(cpu.test_flag(FLAG_ZERO));
        assert!(!cpu.test_flag(FLAG_AUXCARRY));
        assert!(!cpu.test_flag(FLAG_PARITY));
        assert!(cpu.test_flag(FLAG_CARRY));
    }

    #[test]
    fn test_xchg() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.d = 0xAB;
        cpu.e = 0xCD;

        cpu.prep_instr_and_data(0xEB, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.h, 0xAB);
        assert_eq!(cpu.l, 0xCD);
        assert_eq!(cpu.d, 0x12);
        assert_eq!(cpu.e, 0x34);
    }

    #[test]
    fn test_shld() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.h = 0x0AE;
        cpu.l = 0x029;

        cpu.prep_instr_and_data(0x22, 0x01, 0x0A);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);

        assert_eq!(cpu.memory.read(0x0A01).unwrap(), 0x29);
        assert_eq!(cpu.memory.read(0x0A02).unwrap(), 0xAE);
    }

    #[test]
    fn test_rlc_ral() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Test RLC
        cpu.a = 0x0AA;
        cpu.reset_flag(FLAG_CARRY);
        cpu.prep_instr_and_data(0x07, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.a, 0x55);
        assert!(cpu.test_flag(FLAG_CARRY));

        // Test RAL
        cpu.a = 0x0AA;
        cpu.reset_flag(FLAG_CARRY);
        cpu.prep_instr_and_data(0x17, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0x54);
        assert!(cpu.test_flag(FLAG_CARRY));
    }

    #[test]
    fn test_ldax() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Prep our memory
        cpu.memory.write(0x938B, 0xA4).unwrap(); // For BC
        cpu.memory.write(0x13FA, 0xC4).unwrap(); // For DE

        cpu.set_register_pair(Registers::BC, 0x938B);
        cpu.prep_instr_and_data(0x0A, 0x00, 0x00); // LDAX BC
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);
        assert_eq!(cpu.a, 0xA4);

        cpu.set_register_pair(Registers::DE, 0x13FA);
        cpu.prep_instr_and_data(0x1A, 0x00, 0x00); // LDAX DE
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.a, 0xC4);
    }

    #[test]
    fn test_lda() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Prep our memory
        cpu.memory.write(0x025B, 0xFF).unwrap();

        // Call the instruction
        cpu.prep_instr_and_data(0x3A, 0x5B, 0x02); // Load accum with mem value at location 0x025B
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);
    }

    #[test]
    fn test_lhld() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // cpu.current_instruction = Instruction::new(0x2A);
        // Setup the DL and DH values so the address will be appropriate
        cpu.prep_instr_and_data(0x2A, 0x5B, 0x02);

        // Setup our memory *at* that location so we can store the values there in L and H
        cpu.memory.write(0x25B, 0xFF).unwrap();
        cpu.memory.write(0x25C, 0x03).unwrap();

        // Run the opcode
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.l, 0xFF);
        assert_eq!(cpu.h, 0x03);
        assert_eq!(cpu.pc, op + (OPCODE_SIZE * 3));
    }

    #[test]
    fn test_mov() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Test a register to register move (E into B)
        cpu.b = 0x00;
        cpu.e = 0x10;
        cpu.prep_instr_and_data(0x43, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.b, cpu.e);
        assert_eq!(cpu.pc, op + OPCODE_SIZE);

        // Test a register to memory addr move (move B into the memory address located at HL)
        cpu.h = 0x10;
        cpu.l = 0xFF;
        cpu.prep_instr_and_data(0x70, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.memory.read(0x10FF).unwrap(), 0x10);

        // Test a memory addr to register move (move into C the value located in memory at HL)
        cpu.c = 0x00;
        cpu.prep_instr_and_data(0x4E, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.c, 0x10);
    }

    #[test]
    fn test_lxi() {
        // This will test a load into the BC pair
        let mut cpu = CPU::new();
        let op = cpu.pc;
        cpu.prep_instr_and_data(0x01, 0x01, 0x02);

        cpu.run_opcode().unwrap();

        assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);
        assert_eq!(cpu.b, 0x02);
        assert_eq!(cpu.c, 0x01);

        // This will test a load into memory
        cpu.pc = 0;
        cpu.prep_instr_and_data(0x31, 0x01, 0x02);
        cpu.run_opcode().unwrap();

        // SP should be 0x0201
        assert_eq!(cpu.sp, 0x0201);
    }

    #[test]
    fn test_mvi() {
        let mut cpu = CPU::new();
        let op = cpu.pc;
        cpu.prep_instr_and_data(0x3E, 0x01, 0x02);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE * 2);
        assert_eq!(cpu.a, 0x01);
    }
}
