use crate::cpu::CPU;

/// This contains any instructions of the MISC / CONTROL category
/// that need to be implemented within the CPU

#[allow(clippy::unnecessary_wraps)]
impl CPU {
    // OUT D8
    // Would send the contents of accumulator to the device sent
    // as the data portion of this command
    // TODO: If data out is needed, this needs to be finished
    #[allow(clippy::unused_self)]
    pub fn op_out(&self, data: u8) -> Result<(), String> {
        println!("Setting Data Out: {:#04X}", data);
        Ok(())
    }
    // ProgramCounter is incremented and then the CPU enters a
    // STOPPED state and no further activity takes place until
    // an interrupt occurrs
    pub fn op_hlt(&mut self) -> Result<(), String> {
        self.nop(true);
        Ok(())
    }
}

#[cfg(test)]
mod test_misc {
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
}
