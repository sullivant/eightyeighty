#[cfg(test)]
mod tests {
    use crate::{
        bus::Bus, constants::{FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO}, cpu::{CPU, Registers, make_pointer, will_ac}, memory::Memory
    };

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        cpu.nop = false;
        cpu.nop(true);
        assert!(cpu.nop);
    }

    #[test]
    fn test_cpu_default() {
        let mut cpu = CPU::new();
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
        let mut bus: Bus = Bus::new(Memory::new());
        cpu.prep_instr_and_data(&mut bus, 0x76, 0x10, 0x01);

        // our initial PC will be 0x00, so after this test, our DL and DH values will
        // be stored at PC+1 and PC+2, respectively.
        assert_eq!(bus.read(cpu.pc + 1), 0x10);
        assert_eq!(bus.read(cpu.pc + 2), 0x01);

        // This will also setup current_instruction's value to be of the opcode specified
        assert_eq!(cpu.current_instruction.opcode, 0x76);
    }

    #[test]
    fn test_get_data_pair() {
        let mut cpu = CPU::new();
        let mut bus: Bus = Bus::new(Memory::new());
        // Setup PC is 0x00.  So let's set PC+1 (DL) and PC+2 (DH)
        bus.write(cpu.pc + 1, 0x10); // DL
        bus.write(cpu.pc + 2, 0x01); // DH
        assert_eq!(cpu.get_data_pair(&bus), (0x10, 0x01));
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

    #[test]
    fn test_set_register_pair() {
        let mut cpu: CPU = CPU::new();

        cpu.set_register_pair(Registers::BC, 0x1001);
        cpu.set_register_pair(Registers::DE, 0xffaa);
        cpu.set_register_pair(Registers::HL, 0x2010);
        cpu.set_register_pair(Registers::SP, 0x1001);

        assert_eq!(cpu.b, 0x10);
        assert_eq!(cpu.c, 0x01);
        assert_eq!(cpu.d, 0xff);
        assert_eq!(cpu.e, 0xaa);
        assert_eq!(cpu.h, 0x20);
        assert_eq!(cpu.l, 0x10);
        assert_eq!(cpu.sp, 0x1001);
    }

    #[test]
    fn test_reset_flag() {
        let mut cpu = CPU::new();
        cpu.flags = 0b1111_1111;
        cpu.reset_flag(FLAG_SIGN);
        assert_eq!(cpu.flags, 0b0111_1111);

        // Test an already reset flag
        cpu.flags = 0b01111111;
        cpu.reset_flag(FLAG_SIGN);
        assert_eq!(cpu.flags, 0b01111111);

        cpu.flags = 0b11111111;
        cpu.reset_flag(FLAG_SIGN | FLAG_ZERO);
        assert_eq!(cpu.flags, 0b00111111);
    }

    #[test]
    fn test_set_flag() {
        let mut cpu = CPU::new();
        cpu.flags = 0b0;
        cpu.set_flag(FLAG_PARITY);
        assert_eq!(cpu.flags, 0b0100);

        // Test an already set flag
        cpu.set_flag(FLAG_PARITY);
        assert_eq!(cpu.flags, 0b0100);
        cpu.flags = 0b0;

        // Test setting multiple at once
        cpu.set_flag(FLAG_PARITY | FLAG_CARRY);
        assert_eq!(cpu.flags, 0b0000_0101);
    }
}
