use crate::cpu::CPU;
use crate::bus::Bus;

/// This contains any instructions of the MISC / CONTROL category
/// that need to be implemented within the CPU
impl CPU {
    /// OUT D8
    /// Would send the contents of accumulator to the device sent
    /// as the data portion of this command
    /// TODO: If data out is needed, this needs to be finished
    pub fn data_out(&self, bus: &mut Bus, device: u8) -> Result<u8, String> {
        let data = self.a;
        bus.output(device, data);
        // println!("Setting Accumulator value '{data:#04X}' to device: {device:#04X}");
        Ok(self.current_instruction.cycles)
    }

    /// IN
    /// An 8 bit data byte is read from device number (exp) and
    /// replaces the contents of the accumulator
    pub fn data_in(&mut self, bus: &mut Bus, device: u8) -> Result<u8, String> {
        let data = bus.input(device);

        self.a = data;

        Ok(self.current_instruction.cycles)
    }

    /// `ProgramCounter` is incremented and then the CPU enters a
    /// STOPPED state and no further activity takes place until
    /// an interrupt occurrs
    pub fn hlt(&mut self) -> Result<u8, String> {
        self.halted = true;
        Ok(self.current_instruction.cycles)
    }

    /// Enables interrupts
    pub fn ei(&mut self) -> Result<u8, String> {
        self.interrupts_enabled = true;
        Ok(self.current_instruction.cycles)
    }

    /// Disables interrupts
    pub fn di(&mut self) -> Result<u8, String> {
        self.interrupts_enabled = false;
        Ok(self.current_instruction.cycles)
    }
}

#[cfg(test)]
mod tests {
    use crate::{bus::Bus, cpu::{CPU, instructions::Instruction}, memory::Memory};

    #[test]
    fn test_op_hlt() {
        let mut cpu = CPU::new();
        let mut bus: Bus = Bus::new(Memory::new());
        let op = cpu.pc;

        // Setup this instruction
        cpu.current_instruction = Instruction::new(0x76);
        cpu.run_opcode(&mut bus).unwrap();
        assert_eq!(cpu.pc, op + cpu.current_instruction.size);

    }
}
