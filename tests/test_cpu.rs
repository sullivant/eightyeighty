extern crate lib;
use lib::Cpu;

pub const OPCODE_SIZE: usize = 1; // TODO: This needs to be in super/lib/cpu

#[test]
fn test_cpu_default() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x201;
    cpu = Cpu::default();
    assert_eq!(cpu.pc, 0x00);
}

#[test]
fn test_nop() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x00, 0x00, 0x00));
    assert_eq!(cpu.pc, op + OPCODE_SIZE);
}

#[test]
fn test_op_11() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x11, 0x01, 0x02));
    assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);
    assert_eq!(cpu.d, 0x02);
    assert_eq!(cpu.e, 0x01);
}

#[test]
// A gets memory value at memory[DE]
fn test_op_1a() {
    let mut cpu = Cpu::new();
    cpu.memory[0x1122] = 0x56;
    let op = cpu.pc;
    cpu.d = 0x11;
    cpu.e = 0x22;
    cpu.a = 0x00;
    cpu.run_opcode((0x1A, 0x00, 0x00));
    assert_eq!(cpu.a, 0x56);
}

#[test]
fn test_op_21() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x21, 0x01, 0x02));
    assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);
    assert_eq!(cpu.h, 0x02);
    assert_eq!(cpu.l, 0x01);
}

#[test]
fn test_op_31() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.run_opcode((0x31, 0x00, 0x24));
    assert_eq!(cpu.pc, op + OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2400);
}

#[test]
fn test_op_77() {
    let mut cpu = Cpu::new();
    cpu.h = 0x20; // H and L registers specify target location
    cpu.l = 0x01; // in memory to load the value of register A
    cpu.a = 0x45;
    cpu.memory[0x2001] = 0; // Reset it.
    cpu.run_opcode((0x77, 0x00, 0x00));
    assert_eq!(cpu.memory[0x2001], 0x45);
}

#[test]
fn test_op_c3() {
    let mut cpu = Cpu::new();
    cpu.run_opcode((0xC3, 0x01, 0x02));
    assert_eq!(cpu.pc, 0x0201);
}

#[test]
fn test_op_c5() {
    let mut cpu = Cpu::new();
    cpu.c = 0x01;
    cpu.b = 0x02;
    assert_eq!(cpu.sp, 0x00); //Starting stack pointer of 0x00
    cpu.run_opcode((0x31, 0x00, 0x24)); // Set the stack pointer to a reasonable spot
    assert_eq!(cpu.sp, 0x2400);
    let sp = cpu.sp;

    let pc = cpu.pc; // For to check after this opcode runs
    cpu.run_opcode((0xc5, 0x00, 0x00));

    // Assert memory looks good
    assert_eq!(cpu.memory[usize::from(sp - 2)], cpu.c);
    assert_eq!(cpu.memory[usize::from(sp - 1)], cpu.b);

    // Assert sp has been updated
    assert_eq!(cpu.sp, (0x2400 - 2));

    // Assert PC is correct
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_cd() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    cpu.sp = 0x2400;

    // Pretend we are going to CALL addr of 0x53
    cpu.run_opcode((0xCD, 0x03, 0x05));

    // memory should be set now
    assert_eq!(cpu.memory[0x23FF], 0x12 >> 4);
    assert_eq!(cpu.memory[0x23FE], 0x12 & 0x0F);

    // Check stack pointer
    assert_eq!(cpu.sp, 0x2402);

    // Check program counter
    assert_eq!(cpu.pc, (0x0503));
}
