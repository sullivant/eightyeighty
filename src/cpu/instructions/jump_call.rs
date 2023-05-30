use crate::{
    constants::{FLAG_CARRY, OPCODE_SIZE},
    cpu::{make_pointer, CPU},
};

/// This contains any instructions of the JUMP / CALL category
/// that need to be implemented within the CPU

#[allow(clippy::unnecessary_wraps)]
impl CPU {
    /// Performs a JUMP (JMP) - Program execution continues unconditionally <br>
    /// at the memory address made by combining (dh) with (dl) (concatenation) and
    /// then updating the `ProgramCounter` value.
    pub fn jmp(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let ys: u16 = u16::from(dh) << 8;
        let dest: u16 = ys | u16::from(dl);

        self.pc = dest.into();

        Ok(())
    }

    /// If `FLAG_CARRY` is set to 1 this will jump to the address specified
    /// when calling the instruction.  If 0, this will simply carry on to
    /// the next instruction.
    pub fn jc(&mut self, dl: u8, dh: u8) {
        if self.test_flag(FLAG_CARRY) {
            self.current_instruction.size = 0;
            self.pc = make_pointer(dl, dh) as usize;
        } else {
            self.current_instruction.size = OPCODE_SIZE * 3;
        }
    }

    /// Performs a CALL instruction by setting the PC to the next sequential
    /// instruction and then pushes the contents of the PC onto the stack and
    /// then jumps to the address specified in the instruction by setting
    /// the PC to the supplied address.
    pub fn call(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        // Set the PC to the next sequential instruction
        self.pc += OPCODE_SIZE * 3;

        // Save away the current PC's hi/low values onto the stack
        let pc_hi = self.pc >> 8;
        let pc_lo = self.pc & 0xFF;

        match self.push(pc_lo as u8, pc_hi as u8) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "CALL: Unable to push PC {0}, {1} onto stack. error is: {e}",
                    pc_hi, pc_lo
                ))
            }
        };

        // Now do our jump by setting the PC to the supplied address.
        self.pc = make_pointer(dl, dh) as usize;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        constants::{FLAG_CARRY, OPCODE_SIZE},
        cpu::CPU,
    };

    #[test]
    fn test_jc() {
        let mut cpu = CPU::new();
        cpu.pc = 0xBCD2;

        cpu.set_flag(FLAG_CARRY);
        cpu.prep_instr_and_data(0xDA, 0x00, 0x20);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x2000);

        cpu.pc = 0xBCD2;
        cpu.reset_flag(FLAG_CARRY);
        cpu.prep_instr_and_data(0xDA, 0x00, 0x20);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0xBCD2 + (OPCODE_SIZE * 3));
    }

    #[test]
    fn test_call() {
        let mut cpu = CPU::new();
        cpu.pc = 0xBCD2;
        cpu.sp = 0x2000; // Setup a stack pointer
        cpu.prep_instr_and_data(0xCD, 0x20, 0xFA);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0xFA20); // PC should be in the target location

        // Stack should hold the prior "next" SP
        let next_pc = 0xBCD2 + (OPCODE_SIZE * 3);
        let pc_hi = next_pc >> 8;
        let pc_lo = next_pc & 0xFF;
        assert_eq!(pc_hi as u8, cpu.memory.read(0x1FFF).unwrap());
        assert_eq!(pc_lo as u8, cpu.memory.read(0x1FFE).unwrap());
        assert_eq!(cpu.sp, 0x1FFE);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new();
        cpu.prep_instr_and_data(0xC3, 0x03, 0x3C);

        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x3C03);
    }
}
