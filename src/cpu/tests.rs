#[cfg(test)]
use crate::cpu::{will_ac, CPU};

#[test]
fn test_cpu_default() {
    let mut cpu = self::CPU::new();
    cpu.pc = 0x201;
    cpu = CPU::default();
    assert_eq!(cpu.pc, 0x00);
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
    assert_eq!(cpu.memory[cpu.pc+1],0x10);
    assert_eq!(cpu.memory[cpu.pc+2],0x01);
    
    // This will also setup current_instruction's value to be of the opcode specified
    assert_eq!(cpu.current_instruction.opcode,0x76);
}