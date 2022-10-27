#[cfg(test)]
mod tests {
    use crate::cpu::{make_pointer, will_ac, CPU, Registers};

    #[test]
    fn test_cpu_default() {
        let mut cpu = self::CPU::new();
        cpu.pc = 0x201;
        cpu = CPU::default();
        assert_eq!(cpu.pc, 0x00);
    }

    #[test]
    fn test_make_pointer() {
        assert_eq!(make_pointer(0x10, 0xF1), 0xF110);
    }

    #[test]
    fn test_will_ac() {
        assert_eq!(will_ac(62, 34), true);
        assert_eq!(will_ac(0b1111, 1), true);
        assert_eq!(will_ac(2, 4), false);
    }

    #[test]
    fn test_prep_instr_and_data() {
        let mut cpu = CPU::new();
        cpu.prep_instr_and_data(0x76, 0x10, 0x01);

        // our initial PC will be 0x00, so after this test, our DL and DH values will
        // be stored at PC+1 and PC+2, respectively.
        assert_eq!(cpu.memory.read(cpu.pc + 1).unwrap(), 0x10);
        assert_eq!(cpu.memory.read(cpu.pc + 2).unwrap(), 0x01);

        // This will also setup current_instruction's value to be of the opcode specified
        assert_eq!(cpu.current_instruction.opcode, 0x76);
    }

    #[test]
    fn test_get_data_pair() {
        let mut cpu = CPU::new();
        // Setup PC is 0x00.  So let's set PC+1 (DL) and PC+2 (DH)
        cpu.memory.write(cpu.pc + 1, 0x10).unwrap(); // DL
        cpu.memory.write(cpu.pc + 2, 0x01).unwrap(); // DH

        assert_eq!(cpu.get_data_pair().unwrap(), (0x10, 0x01));
    }

    #[test]
    fn test_get_register_pair() {
        let mut cpu = CPU::new();
        cpu.b = 0x10;
        cpu.c = 0x01;
        cpu.d = 0xff;
        cpu.e = 0xaa;
        cpu.h = 0x20;
        cpu.l = 0x10;
        cpu.sp = 0x1011;

        assert_eq!(cpu.get_register_pair(Registers::BC), 0x1001);
        assert_eq!(cpu.get_register_pair(Registers::DE), 0xffaa);
        assert_eq!(cpu.get_register_pair(Registers::HL), 0x2010);
        assert_eq!(cpu.get_register_pair(Registers::SP), 0x1011);
        assert_eq!(cpu.get_register_pair(Registers::A), 0x00);
    }
}
