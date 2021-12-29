pub use super::cpu::Cpu;
use std::fmt;

pub const HEADER: &str = "CYCLE:PC Ins S l h sp SZ0A0P1C (lo,hi) B Command";

pub struct Instr {
    code: String, // The string defining what this this instr is actually doing
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

enum ProgramCounter {
    Next,        // The operation does not use any data
    Two,         // The operation uses only 1 byte of data
    Three,       // The operation uses the full 2 bytes of data
    Jump(usize), // The operation jumps to a point in memory
}

enum Registers {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL, // Used to ref memory locations
}

// Really this just prints stuff to the standard output so we can view details on what is
// happening. Later, it will probably print out more of the registers, etc.
pub fn get_opcode_text(op: (u8, u8, u8)) -> Instr {
    let dl = op.1;
    let dh = op.2;
    match op.0 {
        0x00 => Instr {
            code: "NOP".to_string(),
        },
        0x01 => Instr {
            code: "LXI B".to_string(),
        },
        0x03 => op_03(), // INX BC
        0x04 => Instr {
            code: "INR B".to_string(),
        },
        0x05 => op_05(), // DCR B
        0x06 => op_06(), // MVI B, D8
        0x09 => op_09(), // DAD B (HL = HL + BC)
        0x0A => Instr {
            code: "LDAX BC".to_string(),
        },
        0x0B => Instr {
            code: "DCX BC".to_string(),
        },
        0x0C => Instr {
            code: "INR C".to_string(),
        },
        0x0E => op_0e(), // MVI C, D8
        0x11 => Instr {
            code: "LXI D".to_string(),
        },
        0x13 => op_13(), // INX DE
        0x14 => Instr {
            code: "INR D".to_string(),
        },
        0x15 => Instr {
            code: "DCR D".to_string(),
        },
        0x16 => op_16(), // MVI D
        0x17 => Instr {
            code: "RAL".to_string(),
        },
        0x19 => op_19(), // DAD D (HL = HL + DE)
        0x1A => op_1a(), // LDAX D
        0x1B => Instr {
            code: "DCX DE".to_string(),
        },
        0x1C => Instr {
            code: "INR E".to_string(),
        },
        0x1E => op_1e(), // MVI E
        0x21 => Instr {
            code: "LXI H".to_string(),
        },
        0x23 => op_23(), // INX HL
        0x24 => Instr {
            code: "INR H".to_string(),
        },
        0x25 => Instr {
            code: "DCR H".to_string(),
        },
        0x26 => op_26(), // MVI H, D8
        0x29 => op_29(), // DAD H (HL = HL + HL)
        0x2A => Instr {
            code: "LHLD".to_string(),
        },
        0x2B => Instr {
            code: "DCX HL".to_string(),
        },
        0x2C => Instr {
            code: "INR L".to_string(),
        },
        0x2E => op_2e(), // MVI L, D8
        0x31 => op_31(), // LXI SP, D16
        0x32 => Instr {
            code: "STA (adr)<-A".to_string(),
        },
        0x33 => op_33(), // INX SP
        0x34 => Instr {
            code: "INR (HL)".to_string(),
        },
        0x35 => Instr {
            code: "DCR (HL)".to_string(),
        },
        0x36 => op_36(), // MVI (HL), D8
        0x3B => Instr {
            code: "DCX SP".to_string(),
        },
        0x3C => Instr {
            code: "INR A".to_string(),
        },
        0x3E => op_3e(), // MVI A, D8
        0x40 => Instr {
            code: "MOV B,B".to_string(),
        },
        0x41 => Instr {
            code: "MOV B,C".to_string(),
        },
        0x42 => Instr {
            code: "MOV B,D".to_string(),
        },
        0x43 => Instr {
            code: "MOV B,E".to_string(),
        },
        0x44 => Instr {
            code: "MOV B,H".to_string(),
        },
        0x45 => Instr {
            code: "MOV B,L".to_string(),
        },
        0x46 => Instr {
            code: "MOV B,(HL)".to_string(),
        },
        0x47 => Instr {
            code: "MOV B,A".to_string(),
        },
        0x6F => op_6f(),              // MOV L, A
        0x70 => op_7m(Registers::B),  // MOV M,B
        0x71 => op_7m(Registers::B),  // MOV M,C
        0x72 => op_7m(Registers::B),  // MOV M,D
        0x73 => op_7m(Registers::B),  // MOV M,E
        0x74 => op_7m(Registers::B),  // MOV M,H
        0x75 => op_7m(Registers::B),  // MOV M,L
        0x76 => op_76(),              // HLT 1 (special)
        0x77 => op_7m(Registers::A),  // MOV M,A
        0x78 => op_7a(Registers::B),  // MOV A,B
        0x79 => op_7a(Registers::C),  // MOV A,C
        0x7A => op_7a(Registers::D),  // MOV A,D
        0x7B => op_7a(Registers::E),  // MOV A,E
        0x7C => op_7a(Registers::H),  // MOV A,H
        0x7D => op_7a(Registers::L),  // MOV A,L
        0x7E => op_7a(Registers::HL), // MOV A,M (HL)
        0x7F => op_7a(Registers::A),  // MOV A,A
        0x91 => Instr {
            code: "SUB C".to_string(),
        },
        0x92 => Instr {
            code: "SUB D".to_string(),
        },
        0x97 => Instr {
            code: "SUB A".to_string(),
        },
        0xC0 => Instr {
            code: "RNC".to_string(),
        },
        0xC1 => Instr {
            code: "POP B".to_string(),
        },
        0xC2 => op_c2(dl, dh), // JNZ Addr
        0xC3 => op_c3(dl, dh), // JMP
        0xC4 => Instr {
            code: "CNZ".to_string(),
        },
        0xC5 => op_c5(), // PUSH B
        0xC7 => Instr {
            code: "RST 0".to_string(),
        },
        0xC8 => Instr {
            code: "RC".to_string(),
        },
        0xC9 => op_c9(), // RET
        0xCC => Instr {
            code: "CZ".to_string(),
        },
        0xCF => Instr {
            code: "RST 8".to_string(),
        },
        0xCD => op_cd(dl, dh), // CALL Addr
        0xD0 => Instr {
            code: "RNC".to_string(),
        },
        0xD1 => Instr {
            code: "POP D".to_string(),
        },
        0xD3 => Instr {
            code: "OUT D".to_string(),
        },
        0xD5 => op_d5(), // PUSH D
        0xD7 => Instr {
            code: "RST 2".to_string(),
        },
        0xD4 => Instr {
            code: "CNC Addr".to_string(),
        },
        0xDC => Instr {
            code: "CC Addr".to_string(),
        },
        0xDF => Instr {
            code: "RST 3".to_string(),
        },
        0xE0 => Instr {
            code: "RPO".to_string(),
        },
        0xE1 => Instr {
            code: "POP H".to_string(),
        },
        0xE4 => Instr {
            code: "CPO".to_string(),
        },
        0xE5 => Instr {
            code: "PUSH H".to_string(),
        },
        0xE7 => Instr {
            code: "RST 4".to_string(),
        },
        0xE8 => Instr {
            code: "RPE".to_string(),
        },
        0xEB => Instr {
            code: "XCHG".to_string(),
        },
        0xEC => Instr {
            code: "CPE".to_string(),
        },
        0xEF => Instr {
            code: "RST 5".to_string(),
        },
        0xF4 => op_f4(dl, dh), // CALL if Plus
        0xF5 => Instr {
            code: "PUSH PSW".to_string(),
        },
        0xF0 => Instr {
            code: "RP".to_string(),
        },
        0xF7 => Instr {
            code: "RST 6".to_string(),
        },
        0xFE => Instr {
            code: "CPI".to_string(),
        },
        0xF8 => Instr {
            code: "RM".to_string(),
        },
        0xFF => Instr {
            code: "RST 7".to_string(),
        },
        _ => op_unk(), // UNK
    }
}

pub fn disassemble(cpu: &Cpu, last_pc: usize) -> String {
    let i = get_opcode_text(cpu.last_opcode);
    let dl = cpu.last_opcode.1;
    let dh = cpu.last_opcode.2;

    format!("{:#06X}:{:#06X}   {:#04X} 3  {:#04X},{:#04X},{:#06X}  {:08b}  {:#04X},{:#04X}  {:#04X}  {}",
    cpu.cycle_count, last_pc, cpu.last_opcode.0, cpu.l, cpu.h, cpu.sp, cpu.flags, dl, dh, cpu.b, i.code)
}

// INX BC
fn op_03() -> Instr {
    Instr {
        code: "INX BC".to_string(),
    }
}

// DCR B
fn op_05() -> Instr {
    Instr {
        code: "DCR B".to_string(),
    }
}

fn op_06() -> Instr {
    Instr {
        code: "MVI B, D8".to_string(),
    }
}

// MVI C, D8
fn op_0e() -> Instr {
    Instr {
        code: "MVI C, D8".to_string(),
    }
}

// INX DE
fn op_13() -> Instr {
    Instr {
        code: "INX DE".to_string(),
    }
}

// LDAX DE (A <- $DE)
fn op_1a() -> Instr {
    Instr {
        code: "LDAX DE".to_string(),
    }
}

// INX HL
fn op_23() -> Instr {
    Instr {
        code: "INX HL".to_string(),
    }
}

// MVI H
fn op_26() -> Instr {
    Instr {
        code: "MVI H".to_string(),
    }
}

// DAD H (HL = HL + HI)
fn op_29() -> Instr {
    Instr {
        code: "DAD H".to_string(),
    }
}

fn op_09() -> Instr {
    Instr {
        code: "DAD B".to_string(),
    }
}

fn op_19() -> Instr {
    Instr {
        code: "DAD D".to_string(),
    }
}

// MVI L
fn op_2e() -> Instr {
    Instr {
        code: "MVI L".to_string(),
    }
}

// MVI D
fn op_16() -> Instr {
    Instr {
        code: "MVI D".to_string(),
    }
}

// MVI E
fn op_1e() -> Instr {
    Instr {
        code: "MVI E".to_string(),
    }
}

fn op_31() -> Instr {
    Instr {
        code: "LXI SP, D16".to_string(),
    }
}

// INX SP
fn op_33() -> Instr {
    Instr {
        code: "INX SP".to_string(),
    }
}

// MVI (HL), D8
fn op_36() -> Instr {
    Instr {
        code: "MVI M(HL), D8".to_string(),
    }
}

// MVI A, D8
fn op_3e() -> Instr {
    Instr {
        code: "MVI A, D8".to_string(),
    }
}

// MOV L,A
fn op_6f() -> Instr {
    Instr {
        code: "MOV L, A".to_string(),
    }
}

fn op_76() -> Instr {
    Instr {
        code: "HALT 1".to_string(),
    }
}

// MOV M, Registers::...
fn op_7m(reg: Registers) -> Instr {
    let c: String = match reg {
        Registers::A => "MOV M,A".to_string(),
        Registers::B => "MOV M,B".to_string(),
        Registers::C => "MOV M,C".to_string(),
        Registers::D => "MOV M,D".to_string(),
        Registers::E => "MOV M,E".to_string(),
        Registers::H => "MOV M,H".to_string(),
        Registers::L => "MOV M,L".to_string(),
        _ => "MOV M,M".to_string(),
    };

    Instr { code: c }
}

// MOV M,A
// Address specified by H and L registers.
// Load the value of A into this address in memory.
fn op_7a(reg: Registers) -> Instr {
    let c: String = match reg {
        Registers::A => "MOV A,A".to_string(),
        Registers::B => "MOV A,B".to_string(),
        Registers::C => "MOV A,C".to_string(),
        Registers::D => "MOV A,D".to_string(),
        Registers::E => "MOV A,E".to_string(),
        Registers::H => "MOV A,H".to_string(),
        Registers::L => "MOV A,L".to_string(),
        Registers::HL => "MOV A,(HL)".to_string(),
    };

    Instr { code: c }
}

// JNZ adr
fn op_c2(x: u8, y: u8) -> Instr {
    let ys: u16 = u16::from(y) << 8;
    let dest: u16 = ys | u16::from(x);
    Instr {
        code: format!("JNZ {:#06X}", dest),
    }
}

fn op_c3(x: u8, y: u8) -> Instr {
    let ys: u16 = u16::from(y) << 8;
    let dest: u16 = ys | u16::from(x);
    Instr {
        code: format!("JMP {:#06X}", dest),
    }
}

fn op_c5() -> Instr {
    Instr {
        code: "PUSH B".to_string(),
    }
}

fn op_c9() -> Instr {
    Instr {
        code: "RET".to_string(),
    }
}

fn op_cd(x: u8, y: u8) -> Instr {
    // Tell the program counter where we want to go next
    let ys: u16 = u16::from(y) << 8;
    let adr = usize::from(ys | u16::from(x));

    Instr {
        code: format!("CALL {:#06X}", adr),
    }
}

fn op_d5() -> Instr {
    Instr {
        code: "PUSH D".to_string(),
    }
}

fn op_f4(x: u8, y: u8) -> Instr {
    let ys: u16 = u16::from(y) << 8;
    let adr = usize::from(ys | u16::from(x));

    Instr {
        code: format!("CP {:#06X}", adr),
    }
}

fn op_unk() -> Instr {
    Instr {
        code: "!UNK!".to_string(),
    }
}
