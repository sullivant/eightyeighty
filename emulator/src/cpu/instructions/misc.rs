use crate::cpu::CPU;

/// This contains any instructions of the MISC / CONTROL category
/// that need to be implemented within the CPU

#[allow(clippy::unnecessary_wraps)]
impl CPU {
    /// OUT D8
    /// Would send the contents of accumulator to the device sent
    /// as the data portion of this command
    /// TODO: If data out is needed, this needs to be finished
    pub fn data_out(&self, device: u8) -> Result<u8, String> {
        let data = self.a;
        println!("Setting Accumulator value '{data:#04X}' to device: {device:#04X}");
        Ok(self.current_instruction.cycles)
    }

    /// IN
    /// An 8 bit data byte is read from device number (exp) and
    /// replaces the contents of the accumulator
    pub fn data_in(&mut self, device: u8) -> Result<u8, String> {
        //TODO: This needs to read from a device...
        let data: u8 = 0x00;
        self.a = data;
        println!("Read value '{data:#04X}' from device {device:#04X}");
        Ok(self.current_instruction.cycles)
    }

    /// `ProgramCounter` is incremented and then the CPU enters a
    /// STOPPED state and no further activity takes place until
    /// an interrupt occurrs
    pub fn hlt(&mut self) -> Result<u8, String> {
        self.nop(true);
        Ok(self.current_instruction.cycles)
    }

    /// Enables interrupts
    pub fn ei(&mut self) -> Result<u8, String> {
        self.interrupts = true;
        Ok(self.current_instruction.cycles)
    }

    /// Disables interrupts
    pub fn di(&mut self) -> Result<u8, String> {
        self.interrupts = false;
        Ok(self.current_instruction.cycles)
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{instructions::Instruction, CPU};

    #[test]
    fn test_op_hlt() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        // Setup this instruction
        cpu.current_instruction = Instruction::new(0x76);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, op + cpu.current_instruction.size);

    }
}
