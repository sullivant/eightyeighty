pub use crate::cpu::*;
pub use crate::utils::*;

use std::fmt;

pub const HEADER: &str =
    "CYCLE :PC       Ins  S  l,   h,   sp      SZ0A0P1C  data(l,h)  B    Halt? : Command";

pub fn disassemble(cpu: &Cpu, last_pc: usize) -> String {
    let i = get_opcode_text(cpu.last_opcode);
    let dl = cpu.last_opcode.1;
    let dh = cpu.last_opcode.2;
    format!("{:#06X}:{:#06X}   {:#04X} 3  {:#04X},{:#04X},{:#06X}  {:08b}  {:#04X},{:#04X}  {:#04X} {} : {}",
        cpu.cycle_count, last_pc, cpu.last_opcode.0, cpu.l, cpu.h, cpu.sp, cpu.flags, dl, dh, cpu.b, cpu.nop, i.code)
}

pub struct Instr {
    code: String, // The string defining what this this instr is actually doing
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

pub fn cmd(s: &str) -> Instr {
    Instr {
        code: s.to_string(),
    }
}

// Really this just prints stuff to the standard output so we can view details on what is
// happening. Later, it will probably print out more of the registers, etc.
#[allow(clippy::too_many_lines)]
pub fn get_opcode_text(op: (u8, u8, u8)) -> Instr {
    match op.0 {
        0x00 => cmd("NOP"),
        0x01 => cmd("LXI B"),
        0x02 => cmd("STAX (BC)"),
        0x03 => cmd("INX BC"),
        0x04 => cmd("INR B"),
        0x05 => cmd("DCR B"),
        0x06 => cmd("MVI B"),
        0x07 => cmd("RLC"),
        0x09 => cmd("DAD B"),
        0x0A => cmd("LDAX BC"),
        0x0B => cmd("DCX BC"),
        0x0C => cmd("INR C"),
        0x0D => cmd("DCR C"),
        0x0E => cmd("MVI C"),
        0x0F => cmd("RRC"),
        0x11 => cmd("LXI D"),
        0x12 => cmd("STAX (DE)"),
        0x13 => cmd("INX DE"),
        0x14 => cmd("INR D"),
        0x15 => cmd("DCR D"),
        0x16 => cmd("MVI D"),
        0x17 => cmd("RAL"),
        0x19 => cmd("DAD D"),
        0x1A => cmd("LDAX D"),
        0x1B => cmd("DCX DE"),
        0x1C => cmd("INR E"),
        0x1D => cmd("DCR E"),
        0x1E => cmd("MVI E"),
        0x1F => cmd("RAR"),
        0x21 => cmd("LXI H"),
        0x23 => cmd("INX HL"), // INX HL
        0x24 => cmd("INR H"),
        0x25 => cmd("DCR H"),
        0x26 => cmd("MVI H"), // MVI H, D8
        0x27 => cmd("DAA"),
        0x29 => cmd("DAD H"), // DAD H (HL = HL + HL)
        0x2A => cmd("LHLD"),
        0x2B => cmd("DCX HL"),
        0x2C => cmd("INR L"),
        0x2D => cmd("DCR L"),
        0x2E => cmd("MVI L"),  // MVI L, D8
        0x2F => cmd("CMA"),    // A <= !A
        0x31 => cmd("LXI SP"), // LXI SP, D16
        0x32 => cmd("STA (adr)<-A"),
        0x33 => cmd("INX SP"), // INX SP
        0x34 => cmd("INR (HL)"),
        0x35 => cmd("DCR (HL)"),
        0x36 => cmd("MVI (HL)"), // MVI (HL), D8
        0x37 => cmd("STC"),
        0x39 => cmd("DAD SP"),
        0x3A => cmd("LDA (adr)"),
        0x3B => cmd("DCX SP"),
        0x3C => cmd("INR A"),
        0x3D => cmd("DCR A"),
        0x3E => cmd("MVI A"),
        0x3F => cmd("CMC"), // CY <= !CY
        0x40 => cmd("MOV B,B"),
        0x41 => cmd("MOV B,C"),
        0x42 => cmd("MOV B,D"),
        0x43 => cmd("MOV B,E"),
        0x44 => cmd("MOV B,H"),
        0x45 => cmd("MOV B,L"),
        0x46 => cmd("MOV B,(HL)"),
        0x47 => cmd("MOV B,A"),
        0x48 => cmd("MOV C,B"),
        0x49 => cmd("MOV C,C"),
        0x4A => cmd("MOV C,D"),
        0x4B => cmd("MOV C,E"),
        0x4C => cmd("MOV C,H"),
        0x4D => cmd("MOV C,L"),
        0x4E => cmd("MOV C,(HL)"),
        0x4F => cmd("MOV C,A"),
        0x50 => cmd("MOV D,B"),
        0x51 => cmd("MOV D,C"),
        0x52 => cmd("MOV D,D"),
        0x53 => cmd("MOV D,E"),
        0x54 => cmd("MOV D,H"),
        0x55 => cmd("MOV D,L"),
        0x56 => cmd("MOV D,(HL)"),
        0x57 => cmd("MOV D,A"),
        0x58 => cmd("MOV E,B"),
        0x59 => cmd("MOV E,C"),
        0x5A => cmd("MOV E,D"),
        0x5B => cmd("MOV E,E"),
        0x5C => cmd("MOV E,H"),
        0x5D => cmd("MOV E,L"),
        0x5E => cmd("MOV E,(HL)"),
        0x5F => cmd("MOV E,A"),
        0x60 => cmd("MOV H,B"),
        0x61 => cmd("MOV H,C"),
        0x62 => cmd("MOV H,D"),
        0x63 => cmd("MOV H,E"),
        0x64 => cmd("MOV H,H"),
        0x65 => cmd("MOV H,L"),
        0x66 => cmd("MOV H,(HL)"),
        0x67 => cmd("MOV H,A"),
        0x68 => cmd("MOV L,B"),
        0x69 => cmd("MOV L,C"),
        0x6A => cmd("MOV L,D"),
        0x6B => cmd("MOV L,E"),
        0x6C => cmd("MOV L,H"),
        0x6D => cmd("MOV L,L"),
        0x6E => cmd("MOV L,(HL)"),
        0x6F => cmd("MOV L,A"),
        0x70 => cmd("MOV M,B"),     // MOV M,B
        0x71 => cmd("MOV M,C"),     // MOV M,C
        0x72 => cmd("MOV M,D"),     // MOV M,D
        0x73 => cmd("MOV M,E"),     // MOV M,E
        0x74 => cmd("MOV M,H"),     // MOV M,H
        0x75 => cmd("MOV M,L"),     // MOV M,L
        0x76 => cmd("HLT 1"),       // HLT 1 (special)
        0x77 => cmd("MOV M,A"),     // MOV M,A
        0x78 => cmd("MOV A,B"),     // MOV A,B
        0x79 => cmd("MOV A,C"),     // MOV A,C
        0x7A => cmd("MOV A,D"),     // MOV A,D
        0x7B => cmd("MOV A,E"),     // MOV A,E
        0x7C => cmd("MOV A,H"),     // MOV A,H
        0x7D => cmd("MOV A,L"),     // MOV A,L
        0x7E => cmd("MOV A,M(HL)"), // MOV A,M (HL)
        0x7F => cmd("MOV A,A"),     // MOV A,A
        0x80 => cmd("ADD B"),
        0x81 => cmd("ADD C"),
        0x82 => cmd("ADD D"),
        0x83 => cmd("ADD E"),
        0x84 => cmd("ADD H"),
        0x85 => cmd("ADD L"),
        0x86 => cmd("ADD M(HL)"),
        0x87 => cmd("ADD A"),
        0x88 => cmd("ADC B"),
        0x89 => cmd("ADC C"),
        0x8A => cmd("ADC D"),
        0x8B => cmd("ADC E"),
        0x8C => cmd("ADC H"),
        0x8D => cmd("ADC L"),
        0x8E => cmd("ADC M(HL)"),
        0x8F => cmd("ADC A"),
        0x90 => cmd("SUB B"),
        0x91 => cmd("SUB C"),
        0x92 => cmd("SUB D"),
        0x93 => cmd("SUB E"),
        0x94 => cmd("SUB H"),
        0x95 => cmd("SUB L"),
        0x96 => cmd("SUB (HL)"),
        0x97 => cmd("SUB A"),
        0x98 => cmd("SBB B"),
        0x99 => cmd("SBB C"),
        0x9A => cmd("SBB D"),
        0x9B => cmd("SBB E"),
        0x9C => cmd("SBB H"),
        0x9D => cmd("SBB L"),
        0x9E => cmd("SBB (HL)"),
        0x9F => cmd("SBB A"),
        0xA0 => cmd("ANA B"),
        0xA1 => cmd("ANA C"),
        0xA2 => cmd("ANA D"),
        0xA3 => cmd("ANA E"),
        0xA4 => cmd("ANA H"),
        0xA5 => cmd("ANA L"),
        0xA6 => cmd("ANA (HL)"),
        0xA7 => cmd("ANA A"),
        0xA8 => cmd("XRA B"),
        0xA9 => cmd("XRA C"),
        0xAA => cmd("XRA D"),
        0xAB => cmd("XRA E"),
        0xAC => cmd("XRA H"),
        0xAD => cmd("XRA L"),
        0xAE => cmd("XRA (HL)"),
        0xAF => cmd("XRA A"),
        0xB0 => cmd("ORA B"),
        0xB1 => cmd("ORA C"),
        0xB2 => cmd("ORA D"),
        0xB3 => cmd("ORA E"),
        0xB4 => cmd("ORA H"),
        0xB5 => cmd("ORA L"),
        0xB6 => cmd("ORA (HL)"),
        0xB7 => cmd("ORA A"),
        0xB8 => cmd("CMP B"),
        0xB9 => cmd("CMP C"),
        0xBA => cmd("CMP D"),
        0xBB => cmd("CMP E"),
        0xBC => cmd("CMP H"),
        0xBD => cmd("CMP L"),
        0xBE => cmd("CMP (HL)"),
        0xBF => cmd("CMP A"),
        0xC0 => cmd("RNZ"),
        0xC1 => cmd("POP B"),
        0xC2 => cmd("JNZ Addr"), // JNZ Addr
        0xC3 => cmd("JMP"),      // JMP
        0xC4 => cmd("CNZ"),
        0xC5 => cmd("PUSH B"), // PUSH B
        0xC6 => cmd("ADI"),
        0xC7 => cmd("RST 0"),
        0xC8 => cmd("RC"),
        0xC9 => cmd("RET"), // RET
        0xCA => cmd("JZ Addr"),
        0xCC => cmd("CZ"),
        0xCD => cmd("CALL Addr"), // CALL Addr
        0xCE => cmd("ACI D8"),
        0xCF => cmd("RST 8"),
        0xD0 => cmd("RNC"),
        0xD1 => cmd("POP D"),
        0xD3 => cmd("OUT D"),
        0xD4 => cmd("CNC Addr"),
        0xD5 => cmd("PUSH D"), // PUSH D
        0xD7 => cmd("RST 2"),
        0xDC => cmd("CC Addr"),
        0xDF => cmd("RST 3"),
        0xE0 => cmd("RPO"),
        0xE1 => cmd("POP H"),
        0xE4 => cmd("CPO"),
        0xE5 => cmd("PUSH H"),
        0xE7 => cmd("RST 4"),
        0xE8 => cmd("RPE"),
        0xEB => cmd("XCHG"),
        0xEC => cmd("CPE"),
        0xEF => cmd("RST 5"),
        0xF0 => cmd("RP"),
        0xF4 => cmd("CP"), // CALL if Plus
        0xF5 => cmd("PUSH PSW"),
        0xF7 => cmd("RST 6"),
        0xF8 => cmd("RM"),
        0xFE => cmd("CPI"),
        0xFF => cmd("RST 7"),
        _ => cmd("UNK"), // UNK
    }
}
