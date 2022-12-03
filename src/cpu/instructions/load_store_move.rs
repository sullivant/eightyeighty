use crate::cpu::{make_pointer, Registers, CPU};

/// This contains any instructions of the LOAD / STORE / MOVE category
/// that need to be implemented within the CPU

impl CPU {
    // LXI (target pair), D16
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
                "Register {} is NOT IMPLEMENTED in OP_LXI, Cannot Execute",
                target
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
                    "LHLD: Unable to read for L in memory at {:#04X}",
                    addr
                ))
            }
        };
        addr = addr.overflowing_add(0x01).0;
        self.h = match self.memory.read(addr as usize) {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "LHLD: Unable to read for H in memory at {:#04X}",
                    addr
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
                return Err(format!(
                    "Cannot MOV from unimplemented register: {}",
                    source
                ));
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
                return Err(format!(
                    "Cannot MOV into unimplemented register: {}",
                    source
                ));
            }
        };

        Ok(())
    }

    // Store accumulator direct to location in memory specified
    // by address dhdl
    pub fn op_sta(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let addr: usize = usize::from(u16::from(dh) << 8 | u16::from(dl));
        self.memory.write(addr, self.a)
    }


    // Stores accumulator at memory location of supplied register
    pub fn op_stax(&mut self, reg: Registers) -> Result<(), String> {
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

        Err(format!("Cannot determine location from register pair provided {:#}", reg))
    }


}

#[cfg(test)]
mod tests {
    use crate::constants::OPCODE_SIZE;
    use crate::cpu::CPU;

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
}
