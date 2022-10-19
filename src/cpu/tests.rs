
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
