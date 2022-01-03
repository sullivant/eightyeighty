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

pub fn cmd(s: &str) -> Instr {
    Instr {
        code: s.to_string(),
    }
}

// Really this just prints stuff to the standard output so we can view details on what is
// happening. Later, it will probably print out more of the registers, etc.
pub fn get_opcode_text(op: (u8, u8, u8)) -> Instr {
    match op.0 {
        0x00 => cmd("NOP"),
        0x01 => cmd("LXI B"),
        0x03 => cmd("INX BC"),
        0x04 => cmd("INR B"),
        0x05 => cmd("DCR B"), // DCR B
        0x06 => cmd("MVI B"), // MVI B, D8
        0x09 => cmd("DAD B"), // DAD B (HL = HL + BC)
        0x0A => cmd("LDAX BC"),
        0x0B => cmd("DCX BC"),
        0x0C => cmd("INR C"),
        0x0E => cmd("MVI C"), // MVI C, D8
        0x11 => cmd("LXI D"),
        0x13 => cmd("INX DE"), // INX DE
        0x14 => cmd("INR D"),
        0x15 => cmd("DCR D"),
        0x16 => cmd("MVI D"), // MVI D
        0x17 => cmd("RAL"),
        0x19 => cmd("DAD D"),  // DAD D (HL = HL + DE)
        0x1A => cmd("LDAX D"), // LDAX D
        0x1B => cmd("DCX DE"),
        0x1C => cmd("INR E"),
        0x1E => cmd("MVI E"), // MVI E
        0x21 => cmd("LXI H"),
        0x23 => cmd("INX HL"), // INX HL
        0x24 => cmd("INR H"),
        0x25 => cmd("DCR H"),
        0x26 => cmd("MVI H"), // MVI H, D8
        0x29 => cmd("DAD H"), // DAD H (HL = HL + HL)
        0x2A => cmd("LHLD"),
        0x2B => cmd("DCX HL"),
        0x2C => cmd("INR L"),
        0x2E => cmd("MVI L"),  // MVI L, D8
        0x31 => cmd("LXI SP"), // LXI SP, D16
        0x32 => cmd("STA (adr)<-A"),
        0x33 => cmd("INX SP"), // INX SP
        0x34 => cmd("INR (HL)"),
        0x35 => cmd("DCR (HL)"),
        0x36 => cmd("MVI (HL)"), // MVI (HL), D8
        0x3B => cmd("DCX SP"),
        0x3C => cmd("INR A"),
        0x3E => cmd("MVI A"), // MVI A, D8
        0x40 => cmd("MOV B,B"),
        0x41 => cmd("MOV B,C"),
        0x42 => cmd("MOV B,D"),
        0x43 => cmd("MOV B,E"),
        0x44 => cmd("MOV B,H"),
        0x45 => cmd("MOV B,L"),
        0x46 => cmd("MOV B,(HL)"),
        0x47 => cmd("MOV B,A"),
        0x6F => cmd("MOV L,A"),     // MOV L, A
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
        0x91 => cmd("SUB C"),
        0x92 => cmd("SUB D"),
        0x97 => cmd("SUB A"),
        0xC0 => cmd("RNC"),
        0xC1 => cmd("POP B"),
        0xC2 => cmd("JNZ Addr"), // JNZ Addr
        0xC3 => cmd("JMP"),      // JMP
        0xC4 => cmd("CNZ"),
        0xC5 => cmd("PUSH B"), // PUSH B
        0xC7 => cmd("RST 0"),
        0xC8 => cmd("RC"),
        0xC9 => cmd("RET"), // RET
        0xCC => cmd("CZ"),
        0xCF => cmd("RST 8"),
        0xCD => cmd("CALL Addr"), // CALL Addr
        0xD0 => cmd("RNC"),
        0xD1 => cmd("POP D"),
        0xD3 => cmd("OUT D"),
        0xD5 => cmd("PUSH D"), // PUSH D
        0xD7 => cmd("RST 2"),
        0xD4 => cmd("CNC Addr"),
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
        0xF4 => cmd("CP"), // CALL if Plus
        0xF5 => cmd("PUSH PSW"),
        0xF0 => cmd("RP"),
        0xF7 => cmd("RST 6"),
        0xFE => cmd("CPI"),
        0xF8 => cmd("RM"),
        0xFF => cmd("RST 7"),
        _ => cmd("UNK"), // UNK
    }
}

pub fn disassemble(cpu: &Cpu, last_pc: usize) -> String {
    let i = get_opcode_text(cpu.last_opcode);
    let dl = cpu.last_opcode.1;
    let dh = cpu.last_opcode.2;
    format!("{:#06X}:{:#06X}   {:#04X} 3  {:#04X},{:#04X},{:#06X}  {:08b}  {:#04X},{:#04X}  {:#04X}  {}",
    cpu.cycle_count, last_pc, cpu.last_opcode.0, cpu.l, cpu.h, cpu.sp, cpu.flags, dl, dh, cpu.b, i.code)
}
