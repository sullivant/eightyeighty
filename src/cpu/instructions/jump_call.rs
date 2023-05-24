use crate::cpu::CPU;

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

    use crate::cpu::CPU;

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new();
        cpu.prep_instr_and_data(0xC3, 0x03, 0x3C);

        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x3C03);
    }
}
