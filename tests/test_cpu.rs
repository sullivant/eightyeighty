use lib::Cpu;

#[test]
fn test_cpu_default() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x201;
    cpu = Cpu::default();
    assert_eq!(cpu.pc, 0x00);
}

#[test]
fn test_set_flag() {
    let mut cpu = Cpu::new();
    cpu.flags = 0b0;
    cpu.set_flag(lib::FLAG_PARITY);
    assert_eq!(cpu.flags, 0b0100);

    // Test an already set flag
    cpu.set_flag(lib::FLAG_PARITY);
    assert_eq!(cpu.flags, 0b0100);
    cpu.flags = 0b0;

    // Test setting multiple at once
    cpu.set_flag(lib::FLAG_PARITY | lib::FLAG_CARRY);
    assert_eq!(cpu.flags, 0b0000_0101);
}

#[test]
fn test_reset_flag() {
    let mut cpu = Cpu::new();
    cpu.flags = 0b11111111;
    cpu.reset_flag(lib::FLAG_SIGN);
    assert_eq!(cpu.flags, 0b01111111);

    // Test an already reset flag
    cpu.flags = 0b01111111;
    cpu.reset_flag(lib::FLAG_SIGN);
    assert_eq!(cpu.flags, 0b01111111);

    cpu.flags = 0b11111111;
    cpu.reset_flag(lib::FLAG_SIGN | lib::FLAG_ZERO);
    assert_eq!(cpu.flags, 0b00111111);
}

#[test]
fn test_get_parity() {
    let mut cpu = Cpu::new();
    let mut n: u16 = 0b1100;
    assert_eq!(cpu.get_parity(n), true); // Even 1s, = parity 1
    n = 0b1110;
    assert_eq!(cpu.get_parity(n), false); // Odd 1s, = parity 0
    n = 0b11000011;
    assert_eq!(cpu.get_parity(n), true);

    // Ensure zero is true
    assert_eq!(cpu.get_parity(0b0000), true);
}

#[test]
fn test_get_addr_pointer() {
    let mut cpu = Cpu::new();
    cpu.h = 0x10;
    cpu.l = 0x01;

    let loc = usize::from(u16::from(0x10 as u8) << 8 | u16::from(0x01 as u8));

    assert_eq!(cpu.get_addr_pointer(), loc);
}

#[test]
fn test_get_sign() {
    let mut cpu = Cpu::new();
    assert_eq!(cpu.get_sign(0b11110000), true);
    assert_eq!(cpu.get_sign(0b01110000), false);
    assert_eq!(cpu.get_sign(0b1000u8), false);
    assert_eq!(cpu.get_sign(0b1000 << 4), true);
}

#[test]
fn test_test_flag() {
    let mut cpu = Cpu::new();
    cpu.set_flag(lib::FLAG_ZERO); // Flag zero
    cpu.set_flag(lib::FLAG_PARITY); // Zero and Parity
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), true);
}

#[test]
fn test_update_flags() {
    let mut cpu = Cpu::new();
    // Should update: PARITY (TRUE) SIGN(FALSE) ZERO (TRUE)
    cpu.update_flags(0b0000);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), false);

    cpu.flags = 0;

    // Should update: PARITY (TRUE) SIGN(TRUE) and ZERO (FALSE)
    cpu.update_flags(0b10001000);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), true);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), false);
}

#[test]
fn test_op_00() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x00, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_03() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.b = 0x18;
    cpu.c = 0xff;
    cpu.run_opcode((0x03, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.b, 0x19);
    assert_eq!(cpu.c, 0x00);

    // try again with the overflow protection
    cpu.b = 0xff;
    cpu.c = 0xff;
    cpu.run_opcode((0x03, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, 0x00);
    assert_eq!(cpu.c, 0x00);
}

#[test]
fn test_op_05() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    // A simple decrement
    cpu.b = 0x02;
    cpu.run_opcode((0x05, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, 0x01);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), false);
    cpu.run_opcode((0x05, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, 0x00);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), true);

    // A wrapping decrement
    cpu.b = 0x00;
    cpu.run_opcode((0x05, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, 0xFF);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), true);

    cpu.b = 0x04;
    cpu.run_opcode((0x05, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, 0x03);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
}

#[test]
fn test_op_06() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x06, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.b, 0x01);
}

#[test]
fn test_op_11() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x11, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.d, 0x02);
    assert_eq!(cpu.e, 0x01);
}

#[test]
fn test_op_13() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.d = 0x18;
    cpu.e = 0xff;
    cpu.run_opcode((0x13, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.d, 0x19);
    assert_eq!(cpu.e, 0x00);

    // try again with the overflow protection
    cpu.d = 0xff;
    cpu.e = 0xff;
    cpu.run_opcode((0x13, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.d, 0x00);
    assert_eq!(cpu.e, 0x00);
}

#[test]
// Register A gets memory value at location DE
fn test_op_1a() {
    let mut cpu = Cpu::new();
    cpu.memory[0x1122] = 0x56;
    cpu.d = 0x11;
    cpu.e = 0x22;
    cpu.a = 0x00;
    cpu.run_opcode((0x1A, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x56);
}

#[test]
// Memory at location HL gets register value A
fn test_op_77() {
    let mut cpu = Cpu::new();
    cpu.h = 0x20; // H and L registers specify target location
    cpu.l = 0x01; // in memory to load the value of register A
    cpu.a = 0x45;
    cpu.memory[0x2001] = 0; // Reset it.
    cpu.run_opcode((0x77, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[0x2001], 0x45);
}

#[test]
fn test_op_7m() {
    let mut cpu = Cpu::new();
    cpu.a = 0x01;
    cpu.b = 0x45;
    cpu.c = 0x46;
    cpu.d = 0x47;
    cpu.e = 0x48;
    cpu.h = 0x10;
    cpu.l = 0x01;

    let l = 0x1001;
    cpu.memory[l] = 0xFF;
    let mut op = cpu.pc;

    cpu.run_opcode((0x70, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.b);

    cpu.run_opcode((0x71, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.c);

    cpu.run_opcode((0x72, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.d);

    cpu.run_opcode((0x73, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.e);

    cpu.run_opcode((0x74, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.h);

    cpu.run_opcode((0x75, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.memory[l], cpu.l);
}

#[test]
fn test_op_7a() {
    let mut cpu = Cpu::new();
    cpu.a = 0x01;
    cpu.b = 0x45;
    cpu.c = 0x46;
    cpu.d = 0x47;
    cpu.e = 0x48;
    cpu.h = 0x10;
    cpu.l = 0x01;

    cpu.memory[0x1001] = 0xFF;
    let mut op = cpu.pc;

    cpu.run_opcode((0x78, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.b);

    cpu.run_opcode((0x79, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.c);

    cpu.run_opcode((0x7A, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.d);

    cpu.run_opcode((0x7B, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.e);

    cpu.run_opcode((0x7C, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.h);

    cpu.run_opcode((0x7D, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.l);

    // Sets A = memory at loc (HL)
    cpu.run_opcode((0x7E, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, 0xFF);

    cpu.run_opcode((0x7F, 0x01, 0x02)).unwrap();
    op += lib::OPCODE_SIZE;
    assert_eq!(cpu.pc, op);
    assert_eq!(cpu.a, cpu.a);
}

#[test]
fn test_op_21() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x21, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.h, 0x02);
    assert_eq!(cpu.l, 0x01);
}

#[test]
fn test_op_23() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.h = 0x18;
    cpu.l = 0xff;
    cpu.run_opcode((0x23, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.h, 0x19);
    assert_eq!(cpu.l, 0x00);

    cpu.h = 0x18;
    cpu.l = 0x0F;
    cpu.run_opcode((0x23, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0x18);
    assert_eq!(cpu.l, 0x10);

    // try again with the overflow protection
    cpu.h = 0xff;
    cpu.l = 0xff;
    cpu.run_opcode((0x23, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0x00);
    assert_eq!(cpu.l, 0x00);
}

#[test]
fn test_op_31() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.run_opcode((0x31, 0x00, 0x24)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2400);
}

#[test]
fn test_op_33() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.sp = 0x0018;
    cpu.run_opcode((0x33, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.sp, 0x19);

    // try again with the overflow protection
    cpu.sp = 0xffff;
    cpu.run_opcode((0x33, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.sp, 0x0000);
}

#[test]
fn test_op_36() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.h = 0x20;
    cpu.l = 0x1F;
    cpu.run_opcode((0x36, 0x1A, 0x00)).unwrap();

    assert_eq!(cpu.memory[0x201F], 0x1A);

    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE + 1);
}

#[test]
fn test_op_c2() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.set_flag(lib::FLAG_ZERO);
    cpu.run_opcode((0xC2, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));

    cpu.reset_flag(lib::FLAG_ZERO);
    cpu.run_opcode((0xC2, 0x01, 0x10)).unwrap();
    assert_eq!(cpu.pc, 0x1001);
}

#[test]
fn test_op_c3() {
    let mut cpu = Cpu::new();
    cpu.run_opcode((0xC3, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, 0x0201);
}

#[test]
fn test_op_c5() {
    let mut cpu = Cpu::new();
    cpu.c = 0x01;
    cpu.b = 0x02;
    assert_eq!(cpu.sp, 0x00); //Starting stack pointer of 0x00
    cpu.run_opcode((0x31, 0x00, 0x24)).unwrap(); // Set the stack pointer to a reasonable spot
    assert_eq!(cpu.sp, 0x2400);
    let sp = cpu.sp;

    let pc = cpu.pc; // For to check after this opcode runs
    cpu.run_opcode((0xc5, 0x00, 0x00)).unwrap();

    // Assert memory looks good
    assert_eq!(cpu.memory[usize::from(sp - 2)], cpu.c);
    assert_eq!(cpu.memory[usize::from(sp - 1)], cpu.b);

    // Assert sp has been updated
    assert_eq!(cpu.sp, (0x2400 - 2));

    // Assert PC is correct
    assert_eq!(cpu.pc, pc + lib::OPCODE_SIZE);
}

#[test]
fn test_op_c9() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    cpu.sp = 0x2400;

    // Setup a location to RETurn to on the stack pointer
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    cpu.run_opcode((0xC9, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, 0x1032 + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2402);
}

#[test]
fn test_op_cd() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x18D9;
    cpu.sp = 0x2400;

    // Pretend we are going to CALL addr of 0x0503
    cpu.run_opcode((0xCD, 0x03, 0x05)).unwrap();

    // memory should be set now
    assert_eq!(cpu.memory[0x23FF], 0x18 as u8);
    assert_eq!(cpu.memory[0x23FE], 0xD9 as u8);

    // Check stack pointer
    assert_eq!(cpu.sp, 0x23FE);

    // Check program counter
    assert_eq!(cpu.pc, (0x0503));
}

#[test]
fn test_op_f4() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    let op = cpu.pc;

    // Set a negative test bit register
    cpu.set_flag(lib::FLAG_SIGN);
    // Run opcode with address to NOT jump to
    cpu.run_opcode((0xF4, 0x05, 0x10)).unwrap();
    // PC should be +3 not at the new address
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));

    // Set a positive test bit register
    cpu.reset_flag(lib::FLAG_SIGN);
    // Run the opcode with an address to jump to
    cpu.run_opcode((0xF4, 0x05, 0x10)).unwrap();
    // PC should be the new address.
    assert_eq!(cpu.pc, 0x1005);
}

#[test]
fn test_op_fe() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    cpu.a = 0x05;
    let op = cpu.pc;

    cpu.run_opcode((0xFE, 0x05, 0x10)).unwrap();
}
