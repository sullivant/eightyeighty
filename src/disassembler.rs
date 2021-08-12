struct Instr {
    code: String,         // The string defining what this this instr is actually doing
    size: ProgramCounter, // The size of the program counter "move" after this instr
}

enum ProgramCounter {
    Next,        // The operation does not use any data
    Two,         // The operation uses only 1 byte of data
    Three,       // The operation uses the full 2 bytes of data
    Jump(usize), // The operation jumps to a point in memory
}

// Really this just prints stuff to the standard output so we can view details on what is
// happening. Later, it will probably print out more of the registers, etc.
pub fn disassemble(opcode: (u8, u8, u8), regs: (usize, u16, u8, u8)) {
    let pc = regs.0;
    let h = regs.2;
    let l = regs.3;
    let dl = opcode.1;
    let dh = opcode.2;
    let i = match opcode.0 {
        0x00 => op_00(),       // NOP
        0x06 => op_06(),       // MVI B, D8
        0x11 => op_11(),       // LXI D,D16
        0x1A => op_1a(),       // LDAX D
        0x21 => op_21(),       //	LXI H,D16
        0x23 => op_23(),       // INCX H
        0x31 => op_31(),       // LXI SP, D16
        0x77 => op_77(),       // MOV M,A
        0xC3 => op_c3(dl, dh), // JMP
        0xC5 => op_c5(),       // PUSH B
        0xCD => op_cd(dl, dh), // CALL Addr
        0xD5 => op_d5(),       // PUSH D
        0xE5 => op_e5(),       // PUSH H
        0xF5 => op_f5(),       // PUSH PSW
        _ => op_unk(),         // UNK
    };

    match i.size {
        ProgramCounter::Next => {
            println!(
                "{:#06X}\t{:#04X} 1\t{:#04X},{:#04X}\t\t\t{}",
                pc, opcode.0, l, h, i.code,
            )
        }
        ProgramCounter::Two => {
            println!(
                "{:#06X}\t{:#04X} 2\t{:#04X},{:#04X}\t{:#04X}\t\t{}",
                pc, opcode.0, l, h, dl, i.code
            )
        }
        ProgramCounter::Three => {
            println!(
                "{:#06X}\t{:#04X} 3\t{:#04X},{:#04X}\t{:#04X},{:#04X}\t{}",
                pc, opcode.0, l, h, dl, dh, i.code
            )
        }
        ProgramCounter::Jump(j) => {
            println!(
                "{:#06X}\t{:#04X} 3\t{:#04X},{:#04X}\t{:#04X},{:#04X}\tJMP ${:#06X}",
                pc, opcode.0, l, h, dl, dh, j
            )
        }
    }
}

fn op_00() -> Instr {
    Instr {
        code: "NOP".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_06() -> Instr {
    Instr {
        code: "MVI B, D8".to_string(),
        size: ProgramCounter::Two,
    }
}

// D <- byte 3, E <- byte 2
fn op_11() -> Instr {
    Instr {
        code: "LXI D, D16".to_string(),
        size: ProgramCounter::Three,
    }
}

fn op_13() -> Instr {
    Instr {
        code: "".to_string(),
        size: ProgramCounter::Next,
    }
}

// LDAX DE (A <- $DE)
fn op_1a() -> Instr {
    Instr {
        code: "LDAX D, A".to_string(),
        size: ProgramCounter::Next,
    }
}

// LXI H,D16
fn op_21() -> Instr {
    Instr {
        code: "LXI H, D16".to_string(),
        size: ProgramCounter::Three,
    }
}

// INX HL
fn op_23() -> Instr {
    Instr {
        code: "INX HL".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_31() -> Instr {
    Instr {
        code: "LXI SP, D16".to_string(),
        size: ProgramCounter::Three,
    }
}

// MOV M,A
// Address specified by H and L registers.
// Load the value of A into this address in memory.
fn op_77() -> Instr {
    Instr {
        code: "MOV M,A".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_c3(x: u8, y: u8) -> Instr {
    let ys: u16 = u16::from(y) << 8;
    let dest: u16 = ys | u16::from(x);
    Instr {
        code: format!("JMP {:#06X}", dest),
        size: ProgramCounter::Jump(dest.into()),
    }
}

fn op_c5() -> Instr {
    Instr {
        code: "PUSH B".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_cd(x: u8, y: u8) -> Instr {
    // Tell the program counter where we want to go next
    let ys: u16 = u16::from(y) << 8;
    let adr = usize::from(ys | u16::from(x));

    Instr {
        code: format!("CALL {:#06X}", adr),
        size: ProgramCounter::Three,
    }
}

fn op_d5() -> Instr {
    Instr {
        code: "PUSH D".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_e5() -> Instr {
    Instr {
        code: "PUSH H".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_f5() -> Instr {
    Instr {
        code: "PUSH PSW".to_string(),
        size: ProgramCounter::Next,
    }
}

fn op_unk() -> Instr {
    Instr {
        code: "!UNK!".to_string(),
        size: ProgramCounter::Next,
    }
}
