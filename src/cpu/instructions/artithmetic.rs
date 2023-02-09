use crate::cpu::{make_pointer, Registers, CPU};


impl CPU {
    pub fn op_inx(&mut self, target: Registers) -> Result<(), String> {
        match target {
            Registers::SP | Registers::BC | Registers::DE | Registers::HL => {
                let mut pair: u16 = self.get_register_pair(target);
                pair = pair.overflowing_add(0x01).0;
                self.set_register_pair(target, pair);
            }
            _ => (),
        }
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::constants::OPCODE_SIZE;
    use crate::cpu::CPU;

    #[test]
    fn test_op_inx() {
        let mut cpu = CPU::new();
        let op = cpu.pc;

        cpu.b = 0x18;
        cpu.c = 0xff;
        cpu.prep_instr_and_data(0x03, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.pc, (op + OPCODE_SIZE));
        assert_eq!(cpu.b, 0x19);
        assert_eq!(cpu.c, 0x00);

        // try again with the overflow protection
        cpu.b = 0xff;
        cpu.c = 0xff;
        cpu.prep_instr_and_data(0x03, 0x00, 0x00);
        cpu.run_opcode().unwrap();

        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.c, 0x00);
    }

}

