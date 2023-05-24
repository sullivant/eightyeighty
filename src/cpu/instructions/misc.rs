use crate::cpu::CPU;

/// This contains any instructions of the MISC / CONTROL category
/// that need to be implemented within the CPU

#[allow(clippy::unnecessary_wraps)]
impl CPU {
    /// OUT D8
    /// Would send the contents of accumulator to the device sent
    /// as the data portion of this command
    /// TODO: If data out is needed, this needs to be finished
    #[allow(clippy::unused_self)]
    pub fn data_out(&self, data: u8) -> Result<(), String> {
        println!("Setting Data Out: {data:#04X}");
        Ok(())
    }
    /// `ProgramCounter` is incremented and then the CPU enters a
    /// STOPPED state and no further activity takes place until
    /// an interrupt occurrs
    pub fn hlt(&mut self) -> Result<(), String> {
        self.nop(true);
        Ok(())
    }

    /// Performs a JUMP (JMP) - Program execution continues unconditionally <br>
    /// at the memory address made by combining (dh) with (dl) (concatenation) and
    /// then updating the `ProgramCounter` value.
    pub fn jmp(&mut self, dl: u8, dh: u8) -> Result<(), String> {
        let ys: u16 = u16::from(dh) << 8;
        let dest: u16 = ys | u16::from(dl);

        self.pc = dest.into();

        // Because this is a jump, and a unconditional one, this current instruction's
        // size is actually 0 because we're manually setting the ProgramCounter.  Later on
        // another kind of jump (jump if, etc), may only set size to 0 if conditions are
        // appropriate, etc.
        self.current_instruction.size = 0;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::OPCODE_SIZE;
    use crate::cpu::{instructions::Instruction, CPU};

    #[test]
    fn test_op_hlt() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Setup this instruction
        cpu.current_instruction = Instruction::new(0x76);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);

        // Try to run a tick, PC should not move
        cpu.tick().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE);

        // "unhalt" and see if pc moves next tick
        cpu.nop(false);
        cpu.tick().unwrap();
        assert_eq!(cpu.pc, op + OPCODE_SIZE * 2);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new();
        cpu.prep_instr_and_data(0xC3, 0x03, 0x3C);

        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x3C03);
    }
}
