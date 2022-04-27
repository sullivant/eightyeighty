//#![warn(clippy::all, clippy::pedantic)]

pub use lib::*;

// Lovely aux carry detection
// (a & 0xf) + (b & 0xf) & 0x10 == 0x10
#[test]
fn test_will_ac() {
    let mut cpu = Cpu::new();
    assert_eq!(cpu.will_ac(62, 34), true);
    assert_eq!(cpu.will_ac(0b1111, 1), true);
    assert_eq!(cpu.will_ac(2, 4), false);
}

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
    cpu.flags = 0b1111_1111;
    cpu.reset_flag(lib::FLAG_SIGN);
    assert_eq!(cpu.flags, 0b0111_1111);

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
    let mut n: u16 = 0b1100;
    assert_eq!(lib::get_parity(n), true); // Even 1s, = parity 1
    n = 0b1110;
    assert_eq!(lib::get_parity(n), false); // Odd 1s, = parity 0
    n = 0b11000011;
    assert_eq!(lib::get_parity(n), true);

    // Ensure zero is true
    assert_eq!(lib::get_parity(0b0000), true);
}

#[test]
fn test_get_addr_pointer() {
    let mut cpu = Cpu::new();
    cpu.h = 0x10;
    cpu.l = 0x01;

    let loc = usize::from(u16::from(0x10_u8) << 8 | u16::from(0x01_u8));

    assert_eq!(cpu.get_addr_pointer(), loc);
}

#[test]
fn test_get_sign() {
    assert_eq!(get_sign(0b1111_0000), true);
    assert_eq!(get_sign(0b0111_0000), false);
    assert_eq!(get_sign(0b1000u8), false);
    assert_eq!(get_sign(0b1000 << 4), true);
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
    cpu.update_flags(0b0000, Some(false), Some(false));
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), false);

    cpu.flags = 0;

    // Should update: PARITY (TRUE) SIGN(TRUE) and ZERO (FALSE)
    // And CARRY (TRUE) AUX CARRY(TRUE)
    cpu.update_flags(0b10001000, Some(true), Some(true));
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), true);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), false);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);

    // Check our "aux carry" function for addition
    // (a & 0xf) + (b & 0xf) & 0x10 == 0x10

    cpu.update_flags(0b10001000, Some(true), Some(false));
    cpu.update_flags(
        0b10001000,
        Some(true),
        Some(((62 & 0xf) + (34 & 0xf)) & 0x10 == 0x10),
    );
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);

    // This should skip updating / changing flags because of Some/None
    cpu.update_flags(0b0000, Some(true), Some(true)); // Start with carry and aux carry set
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);
    cpu.update_flags(0b0000, None, None); // Should not touch the carry or aux carry
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);
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
fn test_op_dcr() {
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
fn test_op_inr() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.e = 0x99;
    cpu.run_opcode((0x1C, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.e, 0x9A);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_35() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.h = 0x20;
    cpu.l = 0x00;

    // A simple decrement
    cpu.memory[cpu.get_addr_pointer()] = 0x02;
    cpu.run_opcode((0x35, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[cpu.get_addr_pointer()], 0x01);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), false);
    cpu.run_opcode((0x35, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[cpu.get_addr_pointer()], 0x00);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), true);

    // A wrapping decrement
    cpu.run_opcode((0x35, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[cpu.get_addr_pointer()], 0xFF);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), true);

    cpu.memory[cpu.get_addr_pointer()] = 0x04;
    cpu.run_opcode((0x35, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[cpu.get_addr_pointer()], 0x03);
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
fn test_op_0e() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x0E, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.c, 0x01);
}

#[test]
fn test_op_26() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x26, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.h, 0x01);
}

#[test]
fn test_op_2e() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x2E, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.l, 0x01);
}

#[test]
fn test_op_16() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x16, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.d, 0x01);
}

#[test]
fn test_op_1e() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x1E, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.e, 0x01);
}

#[test]
fn test_op_3e() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.run_opcode((0x3E, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
    assert_eq!(cpu.a, 0x01);
}

#[test]
fn test_op_6f() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.a = 0xF;
    cpu.run_opcode((0x6F, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
    assert_eq!(cpu.l, 0x0F);
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
fn test_op_jz() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.set_flag(lib::FLAG_ZERO);
    cpu.run_opcode((0xCA, 0x01, 0x10)).unwrap();
    assert_eq!(cpu.pc, 0x1001);

    cpu.pc = op;
    cpu.reset_flag(lib::FLAG_ZERO);
    cpu.run_opcode((0xCA, 0x01, 0x02)).unwrap();
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));
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

// POP from the stack to regiser pair HL
// 	L <- (sp); H <- (sp+1); sp <- sp+2
#[test]
fn test_op_e1() {
    let mut cpu = Cpu::new();
    cpu.l = 0x01;
    cpu.h = 0x02;
    assert_eq!(cpu.sp, 0x00); //Starting stack pointer of 0x00
    cpu.run_opcode((0x31, 0x00, 0x24)).unwrap(); // Set the stack pointer to a reasonable spot
    assert_eq!(cpu.sp, 0x2400);
    let sp = cpu.sp;

    let pc = cpu.pc; // For to check after this opcode runs
    cpu.run_opcode((0xE5, 0x00, 0x00)).unwrap();

    // Assert memory looks good
    assert_eq!(cpu.memory[usize::from(sp - 2)], cpu.l);
    assert_eq!(cpu.memory[usize::from(sp - 1)], cpu.h);

    // Assert sp has been updated
    assert_eq!(cpu.sp, (0x2400 - 2));

    // Assert PC is correct
    assert_eq!(cpu.pc, pc + lib::OPCODE_SIZE);
    // Update PC to do the POP portion
    let pc = cpu.pc; // For to check after this opcode runs

    // Reset things
    cpu.l = 0x00;
    cpu.h = 0x00;

    cpu.run_opcode((0xE1, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.l, 0x01);
    assert_eq!(cpu.h, 0x02);
    assert_eq!(cpu.pc, pc + lib::OPCODE_SIZE);
}

#[test]
fn test_op_d5() {
    let mut cpu = Cpu::new();
    cpu.e = 0x01;
    cpu.d = 0x02;
    assert_eq!(cpu.sp, 0x00); //Starting stack pointer of 0x00
    cpu.run_opcode((0x31, 0x00, 0x24)).unwrap(); // Set the stack pointer to a reasonable spot
    assert_eq!(cpu.sp, 0x2400);
    let sp = cpu.sp;

    let pc = cpu.pc; // For to check after this opcode runs
    cpu.run_opcode((0xd5, 0x00, 0x00)).unwrap();

    // Assert memory looks good
    assert_eq!(cpu.memory[usize::from(sp - 2)], cpu.e);
    assert_eq!(cpu.memory[usize::from(sp - 1)], cpu.d);

    // Assert sp has been updated
    assert_eq!(cpu.sp, (0x2400 - 2));

    // Assert PC is correct
    assert_eq!(cpu.pc, pc + lib::OPCODE_SIZE);
}

#[test]
fn test_op_e5() {
    let mut cpu = Cpu::new();
    cpu.l = 0x01;
    cpu.h = 0x02;
    assert_eq!(cpu.sp, 0x00); //Starting stack pointer of 0x00
    cpu.run_opcode((0x31, 0x00, 0x24)).unwrap(); // Set the stack pointer to a reasonable spot
    assert_eq!(cpu.sp, 0x2400);
    let sp = cpu.sp;

    let pc = cpu.pc; // For to check after this opcode runs
    cpu.run_opcode((0xe5, 0x00, 0x00)).unwrap();

    // Assert memory looks good
    assert_eq!(cpu.memory[usize::from(sp - 2)], cpu.l);
    assert_eq!(cpu.memory[usize::from(sp - 1)], cpu.h);

    // Assert sp has been updated
    assert_eq!(cpu.sp, (0x2400 - 2));

    // Assert PC is correct
    assert_eq!(cpu.pc, pc + lib::OPCODE_SIZE);
}

#[test]
fn test_op_ret() {
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

    // Try a return if zero flag is zero
    cpu.pc = 0x12;
    cpu.sp = 0x2400;
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI
    cpu.reset_flag(lib::FLAG_ZERO);
    cpu.run_opcode((0xC0, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, 0x1032 + lib::OPCODE_SIZE * 3);
}

#[test]
fn test_op_rpo() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Setup a location to RETurn to on the stack pointer
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    // try a return with parity NOT odd (parity flag = 1)
    cpu.set_flag(lib::FLAG_PARITY); // EVEN Parity = true(1)
    cpu.run_opcode((0xE0, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    // Try again with partity odd (parity flag = 0)
    cpu.reset_flag(lib::FLAG_PARITY); // ODD parity = false(0)
    cpu.run_opcode((0xE0, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, 0x1032 + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2402);
}

#[test]
fn test_op_rpe() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Setup a location to RETurn to on the stack pointer
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    // Try with parity odd (parity flag = 0)
    cpu.reset_flag(lib::FLAG_PARITY); // ODD parity = false(0)
    cpu.run_opcode((0xE8, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    // try a return with parity NOT odd (parity flag = 1)
    cpu.set_flag(lib::FLAG_PARITY); // EVEN Parity = true(1)
    cpu.run_opcode((0xE8, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, 0x1032 + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2402);
}

#[test]
fn test_op_rm() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Setup a location to RETurn to on the stack pointer
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    cpu.reset_flag(lib::FLAG_SIGN); // true = minus
    cpu.run_opcode((0xF8, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    cpu.set_flag(lib::FLAG_SIGN);
    cpu.run_opcode((0xF8, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, 0x1032 + lib::OPCODE_SIZE * 3);
    assert_eq!(cpu.sp, 0x2402);
}

#[test]
fn test_op_rp() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x12;
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Setup a location to RETurn to on the stack pointer
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    cpu.set_flag(lib::FLAG_SIGN);
    cpu.run_opcode((0xF0, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    cpu.reset_flag(lib::FLAG_SIGN); // true = minus
    cpu.run_opcode((0xF0, 0x00, 0x00)).unwrap();
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
    assert_eq!(cpu.memory[0x23FF], 0x18_u8);
    assert_eq!(cpu.memory[0x23FE], 0xD9_u8);

    // Check stack pointer
    assert_eq!(cpu.sp, 0x23FE);

    // Check program counter
    assert_eq!(cpu.pc, (0x0503));
}

// If the Carry bit is one, a call operation is
// performed to subroutine sub.
#[test]
fn test_cc_addr() {
    let mut cpu = Cpu::new();
    // Setup a current PC value and stack pointer
    cpu.pc = 0x18D9;
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Run it with no carry flag
    cpu.run_opcode((0xDC, 0x03, 0x05)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 3);

    // Reset
    cpu.pc = 0x18D9;

    // Set the carry flag
    cpu.set_flag(lib::FLAG_CARRY);
    cpu.run_opcode((0xDC, 0x03, 0x05)).unwrap();

    // memory should be set now
    assert_eq!(cpu.memory[0x23FF], 0x18_u8);
    assert_eq!(cpu.memory[0x23FE], 0xD9_u8);

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
    cpu.sp = 0x2400;
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
    cpu.a = 0x04;
    let op = cpu.pc;

    cpu.run_opcode((0xFE, 0x05, 0x10)).unwrap();

    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);

    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 2));
}

#[test]
fn test_op_29() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.h = 0x1;
    cpu.l = 0x1;

    cpu.run_opcode((0x29, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0x2);
    assert_eq!(cpu.l, 0x2);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_op_39() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.sp = 0x0101;
    cpu.h = 0x1;
    cpu.l = 0x1;

    cpu.run_opcode((0x39, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0x2);
    assert_eq!(cpu.l, 0x2);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_op_19() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.d = 0x33;
    cpu.e = 0x9F;
    cpu.h = 0xA1;
    cpu.l = 0x7B;

    cpu.run_opcode((0x19, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0xD5);
    assert_eq!(cpu.l, 0x1A);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_op_eb() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.d = 0x33;
    cpu.e = 0x55;
    cpu.h = 0x00;
    cpu.l = 0xFF;

    cpu.run_opcode((0xEB, 0x00, 0x00)).unwrap();

    assert_eq!(cpu.d, 0x00);
    assert_eq!(cpu.e, 0xFF);
    assert_eq!(cpu.h, 0x33);
    assert_eq!(cpu.l, 0x55);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_op_09() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.b = 0x33;
    cpu.c = 0x9F;
    cpu.h = 0xA1;
    cpu.l = 0x7B;

    cpu.run_opcode((0x09, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.h, 0xD5);
    assert_eq!(cpu.l, 0x1A);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_rst() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x1234;
    cpu.sp = 0x2400;

    cpu.run_opcode((0xFF, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.sp, 0x23FE);
    assert_eq!(cpu.memory[0x23FE], 0x12); // High half
    assert_eq!(cpu.memory[0x23FF], 0x34); // Low half
    assert_eq!(cpu.pc, 0x38);
}

#[test]
fn test_sta() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.a = 0x12;

    cpu.run_opcode((0x32, 0x14, 0x59)).unwrap();
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));
    assert_eq!(cpu.memory[0x5914], cpu.a);
}

#[test]
fn test_sub() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.a = 0x12;
    cpu.c = 0x02;

    cpu.run_opcode((0x91, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
    assert_eq!(cpu.a, 0x10);

    cpu.a = 0x3E;
    cpu.run_opcode((0x97, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), true);

    cpu.memory[0x2400] = 0x01;
    cpu.h = 0x24;
    cpu.l = 0x00;
    cpu.a = 0x0B;
    cpu.run_opcode((0x96, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x0A);
}

#[test]
fn test_sbb() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.a = 0x04;
    cpu.l = 0x02;

    cpu.set_flag(lib::FLAG_CARRY);

    cpu.run_opcode((0x9D, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x01);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), false);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), false);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);
}

#[test]
fn test_lhld() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.memory[0x25B] = 0xFF;
    cpu.memory[0x25C] = 0x03;
    cpu.run_opcode((0x2A, 0x5B, 0x02)).unwrap();
    assert_eq!(cpu.l, 0xFF);
    assert_eq!(cpu.h, 0x03);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));
}

#[test]
fn test_dcx() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.d = 0x20;
    cpu.e = 0x00;
    cpu.sp = 0x1234;

    cpu.run_opcode((0x1B, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.d, 0x1F);
    assert_eq!(cpu.e, 0xFF);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));

    cpu.run_opcode((0x3B, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.sp, 0x1233);
}

#[test]
fn test_ral() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x0B5;
    cpu.run_opcode((0x17, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x6A);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
}

#[test]
fn test_rlc() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0xF2;
    cpu.run_opcode((0x07, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0xE5);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
}

#[test]
fn test_rar() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x6A;
    cpu.set_flag(lib::FLAG_CARRY);
    cpu.run_opcode((0x1F, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0xB5);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
}

#[test]
fn test_rrc() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0xF2;
    cpu.run_opcode((0x0F, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x79);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
}

#[test]
fn test_adc() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x42;
    cpu.b = 0x3D;
    cpu.set_flag(lib::FLAG_CARRY);
    // Add the register B to the Accum with the Carry bit, too
    cpu.run_opcode((0x88, 0x00, 000)).unwrap();
    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_add() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x6C;
    cpu.d = 0x2E;
    cpu.set_flag(lib::FLAG_CARRY);
    // Add the register B to the Accum with the Carry bit, too
    cpu.run_opcode((0x82, 0x00, 000)).unwrap();
    assert_eq!(cpu.a, 0x9A);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), true);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));
}

#[test]
fn test_stax() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.memory[0x3F16] = 0x00; // Reset
    cpu.a = 0x20; // This value will be stored at mem loc BC, below
    cpu.b = 0x3F;
    cpu.c = 0x16;

    cpu.run_opcode((0x02, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[0x3F16], 0x20);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE));

    cpu.memory[0x3F16] = 0x00; // Reset
    cpu.a = 0x20; // This value will be stored at mem loc BC, below
    cpu.d = 0x3F;
    cpu.e = 0x16;

    cpu.run_opcode((0x12, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.memory[0x3F16], 0x20);
}

#[test]
fn test_make_pointer() {
    assert_eq!(make_pointer(0x00, 0x03), 0x0300);
}

#[test]
fn test_lda() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x00;
    cpu.memory[0x0300] = 0x3F; // A value in a spot in memory

    cpu.run_opcode((0x3A, 0x00, 0x03)).unwrap();
    assert_eq!(cpu.a, 0x3F);
    assert_eq!(cpu.pc, op + (lib::OPCODE_SIZE * 3));
}

#[test]
fn test_op_cmc() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    // Reset the carry flag
    cpu.set_flag(lib::FLAG_CARRY);
    cpu.run_opcode((0x3F, 0x00, 000)).unwrap();
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    // Set it if it is false
    cpu.run_opcode((0x3F, 0x00, 000)).unwrap();
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
}

#[test]
fn test_op_cma() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x51;
    cpu.run_opcode((0x2F, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0xAE);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_stc() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.reset_flag(lib::FLAG_CARRY);
    cpu.run_opcode((0x37, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_daa() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    // Setup the accum with 0x9B and reset both carry bits
    cpu.a = 0x9b;
    cpu.reset_flag(lib::FLAG_AUXCARRY);
    cpu.reset_flag(lib::FLAG_CARRY);

    cpu.run_opcode((0x27, 0x00, 0x00)).unwrap();

    assert_eq!(cpu.a, 0x01);
    assert!(cpu.test_flag(lib::FLAG_CARRY));
    assert!(cpu.test_flag(lib::FLAG_AUXCARRY));
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_mov() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.b = 0x00;
    cpu.h = 0x20;
    cpu.l = 0x10;
    cpu.c = 0x00;
    cpu.memory[0x2010] = 0x11;

    cpu.run_opcode((0x44, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.b, cpu.h);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    cpu.run_opcode((0x4E, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.c, 0x11);
}

#[test]
fn test_op_hlt() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.run_opcode((0x76, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    // Try to run a tick, PC should not move
    cpu.tick().unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    // "unhalt" and see if pc moves next tick
    cpu.set_nop(false);
    cpu.tick().unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);
}

#[test]
fn test_get_flag() {
    let mut cpu = Cpu::new();

    cpu.reset_flag(lib::FLAG_CARRY);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.get_flag(lib::FLAG_CARRY), 0);

    cpu.set_flag(lib::FLAG_CARRY);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.get_flag(lib::FLAG_CARRY), 1);
}

#[test]
fn test_op_ana() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0xFC;
    cpu.c = 0x0F;

    cpu.run_opcode((0xA1, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_xra() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0xFC;

    // Should zero out the A register
    cpu.run_opcode((0xAF, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    cpu.a = 0xFF;
    cpu.b = 0b0000_1010;
    cpu.run_opcode((0xA8, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0b1111_0101);
}

#[test]
fn test_op_ora() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x33;
    cpu.c = 0x0F;
    cpu.set_flag(lib::FLAG_CARRY);

    // Should zero out the A register
    cpu.run_opcode((0xB1, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x3F);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_cmp() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.a = 0x0A;
    cpu.e = 0x05;
    cpu.set_flag(lib::FLAG_CARRY);

    cpu.run_opcode((0xBB, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x0A);
    assert_eq!(cpu.e, 0x05);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);

    cpu.a = 0x02;
    cpu.run_opcode((0xBB, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x02);
    assert_eq!(cpu.e, 0x05);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);

    cpu.a = !0x1B;
    cpu.run_opcode((0xBB, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.a, !0x1B);
    assert_eq!(cpu.e, 0x05);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
}

#[test]
fn test_op_pop() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;
    cpu.sp = 0x2400;

    // Setup some memory values to pop into our pair
    cpu.memory[usize::from(cpu.sp)] = 0x32; // LO
    cpu.memory[usize::from(cpu.sp + 1)] = 0x10; // HI

    cpu.run_opcode((0xC1, 0x00, 0x00)).unwrap();
    assert_eq!(cpu.c, 0x32);
    assert_eq!(cpu.b, 0x10);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE);
}

#[test]
fn test_op_adi() {
    let mut cpu = Cpu::new();

    // Set the accumulator to 0x14
    cpu.run_opcode((0x3E, 0x14, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x14);

    let op = cpu.pc;
    cpu.run_opcode((0xC6, 0x42, 0x00)).unwrap();

    // Accumulator should now be 0x56 (0x14 + 0x42 = 0x56)
    assert_eq!(cpu.a, 0x56);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), false);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), false);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);

    // Bring us back to the original accumulator value
    cpu.run_opcode((0xC6, 0xBE, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x14);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_AUXCARRY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_PARITY), true);
    assert_eq!(cpu.test_flag(lib::FLAG_SIGN), false);
    assert_eq!(cpu.test_flag(lib::FLAG_ZERO), false);
}

#[test]
fn test_op_aci() {
    let mut cpu = Cpu::new();
    let op = cpu.pc;

    cpu.run_opcode((0x3E, 0x56, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x56);
    assert_eq!(cpu.pc, op + lib::OPCODE_SIZE * 2);

    cpu.reset_flag(lib::FLAG_CARRY);
    cpu.run_opcode((0xCE, 0xBE, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x14);
    assert_eq!(cpu.test_flag(lib::FLAG_CARRY), true);

    // Now, let's do one with a carry flag
    cpu.run_opcode((0xCE, 0x42, 0x00)).unwrap();
    assert_eq!(cpu.a, 0x57);
}
