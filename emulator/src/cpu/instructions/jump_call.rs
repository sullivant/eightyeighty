use crate::{
    constants::{FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO},
    cpu::{make_pointer, CPU},
};

/// This contains any instructions of the JUMP / CALL category
/// that need to be implemented within the CPU

#[allow(clippy::unnecessary_wraps)]
impl CPU {
    /// The contents of the program counter (16bit)
    /// are pushed onto the stack, providing a return address for
    /// later use by a RETURN instruction.
    /// Program execution continues at memory address:
    /// `OOOOOOOO_OOEXPOOOB`
    pub fn rst(&mut self, loc: u8) -> Result<u8, String> {
        let dl = (self.pc as u16 & 0xFF) as u8;
        let dh = (self.pc as u16 >> 8) as u8;
        match self.push(dl, dh) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Jump to the location specified in the opcode.  Example:
        // OP 0xD7 is "RST 2" so the destination ends up being
        // 00000000_00010000 because "EXP" is 010 (2).
        self.jmp(loc << 3, 0x00)
    }

    /// If the Parity bit is zero (indicating odd parity), a
    /// return is performed
    pub fn rpo(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_PARITY) {
            Ok(5)
        } else {
            self.ret()?;
            Ok(11)
        }
    }

    /// If the Parity bit is one (indicating even parity), a
    /// return is performed
    pub fn rpe(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_PARITY) {
            self.ret()?;
            Ok(11)
        } else {
            Ok(5)
        }
    }

    /// If the Sign bit is one (indicating a minus result, a
    /// return is performed
    pub fn rm(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_SIGN) {
            self.ret()?;
            Ok(11)
        } else {
            Ok(5)
        }
    }

    /// If the Sign bit is zero, a return is performed
    pub fn rp(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_SIGN) {
            Ok(5)
        } else {
            self.ret()?;
            Ok(11)
        }
    }

    /// If the Carry bit is one, a return operation is performed
    pub fn rc(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_CARRY) {
            self.ret()?;
            Ok(11)
        } else {
            Ok(5)
        }
    }

    // If the Carry bit is zero, a return operation is performed
    pub fn rnc(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_CARRY) {
            Ok(5)
        } else {
            self.ret()?;
            Ok(11)
        }

    }

    /// If the Zero bit is one, a return operation is performed
    pub fn rz(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_ZERO) {
            self.ret()?;
            Ok(11)
        } else {
            Ok(5)
        }
    }

    /// If the Zero bit is zero, a return operation is performed
    pub fn rnz(&mut self) -> Result<u8, String> {
        if self.test_flag(FLAG_ZERO) {
            Ok(5)
        } else {
            self.ret()?;
            Ok(11)
        }
    }

    /// Performs an immediate return command
    pub fn ret(&mut self) -> Result<u8, String> {
        // RET (PC.lo <- (sp); PC.hi<-(sp+1); SP <- SP+2)
        let pc_lo = self.memory.read(usize::from(self.sp)).unwrap_or(0);
        let pc_hi = self.memory.read(usize::from(self.sp + 1)).unwrap_or(0);

        self.sp += 2;

        // And do an immediate jump
        self.jmp(pc_lo, pc_hi)?;

        Ok(self.current_instruction.cycles)
    }

    /// Performs a JUMP (JMP) - Program execution continues unconditionally <br>
    /// at the memory address made by combining (dh) with (dl) (concatenation) and
    /// then updating the `ProgramCounter` value.
    pub fn jmp(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        self.pc = make_pointer(dl, dh) as usize;

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_CARRY` is set to 1 this will jump to the address specified
    /// when calling the instruction.
    pub fn jc(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_CARRY) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_CARRY` is set to 0 this will jump to the address specified
    /// when calling the instruction.
    pub fn jnc(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if !self.test_flag(FLAG_CARRY) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_ZERO` is set to 1 this will jump to the address specified
    /// when calling the instruction.
    pub fn jz(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_ZERO) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_ZERO` is set to 0 this will jump to the address specified
    /// when calling the instruction.
    pub fn jnz(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if !self.test_flag(FLAG_ZERO) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_SIGN` is set to 1 this will jump to the address specified
    /// when calling the instruction.
    pub fn jm(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_SIGN) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_SIGN` is set to 0 this will jump to the address specified
    /// when calling the instruction.
    pub fn jp(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if !self.test_flag(FLAG_SIGN) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_PARITY` is set to 1 this will jump to the address specified
    /// when calling the instruction.
    pub fn jpe(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_PARITY) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If `FLAG_PARITY` is set to 0 this will jump to the address specified
    /// when calling the instruction.
    pub fn jpo(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if !self.test_flag(FLAG_PARITY) {
            return self.jmp(dl, dh);
        }

        Ok(self.current_instruction.cycles)
    }

    /// If the Carry bit is one, a call operation is performed
    pub fn cc(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_CARRY) {
            self.call(dl, dh)?;
            Ok(17)
        } else {
            Ok(11)
        }
    }

    /// If the Carry bit is zero, a call operation is performed
    pub fn cnc(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_CARRY) { 
            Ok(11)
        } else {
            self.call(dl, dh)?;
            Ok(17)
        }
    }

    /// If the Zero bit is one, a call is performed
    pub fn cnz(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_ZERO) {
            self.call(dl, dh)?;
            Ok(17)
        } else {
            Ok(11)
        }
    }

    /// If the Zero bit is zero, a call is performed
    pub fn cz(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_ZERO) {
            Ok(11)
        } else {
            self.call(dl, dh)?;
            Ok(17)
        }
    }

    /// If the sign bit is one, a call is performed
    pub fn cm(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_SIGN) {
            self.call(dl, dh)?;
            Ok(17)
        } else {
            Ok(11)
        }
    }

    /// If the sign bit is zero, a call is performed
    pub fn cp(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_SIGN) {
            Ok(11)
        } else {
            self.call(dl, dh)?;
            Ok(17)
        }
    }

    /// If the parity bit is one, a call is performed
    pub fn cpe(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_PARITY) {
            self.call(dl, dh)?;
            Ok(17)
        } else {
            Ok(11)
        }
    }

    /// If the parity bit is zero, a call is performed
    pub fn cpo(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        if self.test_flag(FLAG_PARITY) {
            Ok(11)
        } else {
            self.call(dl, dh)?;
            Ok(17)
        }
    }

    /// Contents of the H regsiter replace the 8MSB of the PC and the contents
    /// of the L register replace the 8LSB of the PC.  Program execution continues
    /// at the new location of the PC.  Basically a "jump to the HL register"
    pub fn pchl(&mut self) -> Result<u8, String> {
        self.jmp(self.l, self.h)
    }

    /// Performs a CALL instruction by setting the PC to the next sequential
    /// instruction and then pushes the contents of the PC onto the stack and
    /// then jumps to the address specified in the instruction by setting
    /// the PC to the supplied address.
    pub fn call(&mut self, dl: u8, dh: u8) -> Result<u8, String> {
        // Set the PC to the next sequential instruction
        self.pc += self.current_instruction.size;

        // Save away the current PC's hi/low values onto the stack
        let pc_hi = self.pc >> 8;
        let pc_lo = self.pc & 0xFF;

        match self.push(pc_lo as u8, pc_hi as u8) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "CALL: Unable to push PC {pc_hi}, {pc_lo} onto stack. error is: {e}"
                ))
            }
        }

        // Now do our jump by setting the PC to the supplied address.
        self.pc = make_pointer(dl, dh) as usize;

        Ok(self.current_instruction.cycles)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        constants::{FLAG_CARRY, OPCODE_SIZE},
        cpu::CPU,
    };

    #[test]
    fn test_pchl() {
        let mut cpu = CPU::new();
        cpu.h = 0x41;
        cpu.l = 0x3E;

        cpu.prep_instr_and_data(0xE9, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x413E);
    }

    #[test]
    fn test_rst() {
        let mut cpu = CPU::new();
        cpu.pc = 0xBCD2;
        cpu.sp = 0x2000;

        cpu.prep_instr_and_data(0xC7, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x00);

        cpu.prep_instr_and_data(0xDF, 0x00, 0x00);
        cpu.run_opcode().unwrap();
        assert_eq!(cpu.pc, 0x03 << 3);
    }

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
