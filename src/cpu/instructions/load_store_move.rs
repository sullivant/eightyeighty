use crate::cpu::{Registers, CPU};

/// This contains any instructions of the LOAD / STORE / MOVE category
/// that need to be implemented within the CPU

impl CPU {
    // LXI (target pair), D16
    pub fn lxi(&mut self, target: Registers, dl: u8, dh: u8) -> Result<(), String> {
        match target {
            Registers::BC => {
                self.b = dh;
                self.e = dl;
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
                self.sp = u16::from(dh) << 8 | u16::from(dl);
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
}
