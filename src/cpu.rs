use std::fmt;
use std::fs::File;
use std::io::prelude::*;

pub const RAM_SIZE: usize = 0xFFFF;

pub enum ProgramCounter {
    Next,        // The operation does not use any data
    Two,         // The operation uses only 1 byte of data
    Three,       // The operation uses the full 2 bytes of data
    Jump(usize), // The operation jumps to a point in memory
}

#[derive(Clone, Copy)]
pub enum Registers {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,  // A register pair
    DE,  // A register pair
    HL,  // A register pair, used to reference memory locations
    SP,  // Stack pointer
    PSW, // Program Status Word
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Registers::A => write!(f, "A"),
            Registers::B => write!(f, "B"),
            Registers::C => write!(f, "C"),
            Registers::D => write!(f, "D"),
            Registers::E => write!(f, "E"),
            Registers::H => write!(f, "H"),
            Registers::L => write!(f, "L"),
            Registers::BC => write!(f, "BC"),
            Registers::DE => write!(f, "DE"),
            Registers::HL => write!(f, "HL"),
            Registers::SP => write!(f, "SP"),
            Registers::PSW => write!(f, "PSW"),
        }
    }
}

#[derive(Clone)]
pub struct Cpu {
    // Memory
    pub memory: [u8; RAM_SIZE],

    // Registers
    pub pc: usize, // Program Counter
    pub sp: u16,   // Stack Pointer
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flags Z,S,P,AC
    pub flags: u8,

    // A flag that indicates we wish to print human readable command references
    pub disassemble: bool,
    // A flag to indicate that we do not wish to execute, probably just printing disassembly
    pub nop: bool,

    pub cycle_count: usize,        // Cycle count
    pub last_opcode: (u8, u8, u8), // Just a record of the last opcode.
    pub next_opcode: (u8, u8, u8), // Next opcode we are running.
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CYCLES:{:#08X} PC:{:#06X} SP:{:#06X}\nA:{:#06X}\nB:{:#04X} C:{:#04X}\nD:{:#04X} E:{:#04X}\nH:{:#04X} L:{:#04X}\nsp $[{:#06X}]={:#04X} sp+1 $[{:06X}]={:#04X}",
            self.cycle_count, self.pc, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l,self.sp,self.memory[usize::from(self.sp)],self.sp+1,self.memory[usize::from(self.sp+1)]
        )
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            memory: [0; RAM_SIZE],
            pc: 0x00,
            sp: 0x00,
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            flags: 0x02, // 00000010 is the default starting point
            disassemble: false,
            nop: false,
            cycle_count: 0x00,
            last_opcode: (0, 0, 0),
            next_opcode: (0, 0, 0),
        }
    }

    // Returns a usize location in memory designed by the H and L registers
    pub fn get_addr_pointer(&mut self) -> usize {
        usize::from(u16::from(self.h) << 8 | u16::from(self.l))
    }

    pub fn get_registers(&self) -> (&usize, &u16, &u8, &u8, &u8) {
        (&self.pc, &self.sp, &self.h, &self.l, &self.b)
    }

    // Returns a paired register such as HL or BC.
    // Pass to the function the beginning register for the pair
    // Returned value will be a u16 value
    pub fn get_register_pair(&self, register: Registers) -> u16 {
        match register {
            Registers::BC => u16::from(self.b) << 8 | u16::from(self.c),
            Registers::DE => u16::from(self.d) << 8 | u16::from(self.e),
            Registers::HL => u16::from(self.h) << 8 | u16::from(self.l),
            Registers::SP => self.sp,
            _ => 0_u16,
        }
    }

    // Sets a register pair if appropriate
    pub fn set_register_pair(&mut self, register: Registers, val: u16) {
        let h: u8 = (val >> 8) as u8;
        let l: u8 = (val & 0xff) as u8;
        match register {
            Registers::BC => {
                self.b = h;
                self.c = l;
            }
            Registers::DE => {
                self.d = h;
                self.e = l;
            }
            Registers::HL => {
                self.h = h;
                self.l = l;
            }
            Registers::SP => {
                self.sp = val;
            }
            _ => (),
        };
    }

    // Returns the current flag values
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    // Returns true if a flag is set
    pub fn test_flag(&mut self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    // Sets a flag using a bitwise OR operation
    // Mask of 2 (00100) with a value of 1 = 00100
    // if flags = 10010 new value will be 10110
    pub fn set_flag(&mut self, mask: u8) {
        self.flags |= mask;
    }

    // Resets a flag using bitwise AND operation
    // Mask of 2 (00100) with a value of 0 = 11011
    // if flags = 11111 new value will be 11011
    pub fn reset_flag(&mut self, mask: u8) {
        self.flags &= !mask;
    }

    // Computes and sets the mask of flags for a supplied value
    // sets flags: Zero, Sign, Parity, Carry, and Auxiliary Carry
    pub fn update_flags(&mut self, val: u8, overflow: bool, aux_carry: bool) {
        match val == 0 {
            true => self.set_flag(super::FLAG_ZERO),
            false => self.reset_flag(super::FLAG_ZERO),
        };

        match self.get_sign(val) {
            true => self.set_flag(super::FLAG_SIGN), // A negative number
            false => self.reset_flag(super::FLAG_SIGN), // A positive number
        };

        match self.get_parity(val.into()) {
            true => self.set_flag(super::FLAG_PARITY),
            false => self.reset_flag(super::FLAG_PARITY),
        };

        match overflow {
            true => self.set_flag(super::FLAG_CARRY),
            false => self.reset_flag(super::FLAG_CARRY),
        };

        match aux_carry {
            true => self.set_flag(super::FLAG_AUXCARRY),
            false => self.reset_flag(super::FLAG_AUXCARRY),
        };
    }

    // If number of ones in a number's binary representation is even,
    // parity flag is TRUE (1) else it is FALSE (0)
    pub fn get_parity(&mut self, v: u16) -> bool {
        v.count_ones() % 2 == 0
    }

    // Returns true if MSB = 1
    pub fn get_sign(&mut self, x: u8) -> bool {
        (0b10000000 & x) != 0
    }

    pub fn set_disassemble(&mut self, d: bool) {
        self.disassemble = d;
    }

    pub fn set_nop(&mut self, n: bool) {
        self.nop = n;
    }

    // Load the ROM file into memory, starting at start_index
    // Returns a tuple containing the index we started at and where we
    // actually finished at.
    pub fn load_rom(
        &mut self,
        file: String,
        start_index: usize,
    ) -> Result<(usize, usize), std::io::Error> {
        let rom = File::open(file)?;
        let mut last_idx: usize = 0;
        for (i, b) in rom.bytes().enumerate() {
            self.memory[start_index + i] = b.unwrap();
            last_idx = i;
        }
        Ok((start_index, start_index + last_idx + 1))
    }

    // Gathers a word from memory based on program counter location,
    // then passes it along to the run_opcode() function
    // On successful tick, returns the program counter value that was run
    // On unsuccessful tick, returns an error
    pub fn tick(&mut self) -> Result<usize, String> {
        let opcode = self.read_opcode();
        self.last_opcode = opcode;
        let this_pc = self.pc;

        self.cycle_count += 1;

        match self.run_opcode(opcode) {
            Ok(_) => {
                self.next_opcode = self.read_opcode();
                Ok(this_pc)
            }
            Err(e) => Err(e),
        }
    }

    // Reads an instruction at ProgramCounter
    // Returns the following two bytes as potential "data" for the instruction.
    // If the two bytes are out of range they will return 0x00
    pub fn read_opcode(&mut self) -> (u8, u8, u8) {
        let o = match self.memory.get(self.pc) {
            Some(&v) => v,
            None => 0,
        };
        let x = match self.memory.get(self.pc + 1) {
            Some(&v) => v,
            None => 0,
        };
        let y = match self.memory.get(self.pc + 2) {
            Some(&v) => v,
            None => 0,
        };
        (o, x, y)
    }

    // This will parse the opcode, printing a disassembly if asked
    //
    // An opcode consists of:
    //  Instruction (1 byte)
    //  Data (1 or 2 bytes) depending on opcode.  Little endian.
    //
    // It will also return ERROR if the opcode was not recognized
    pub fn run_opcode(&mut self, opcode: (u8, u8, u8)) -> Result<(), String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        // D8 = 8 bits (1st byte = y)
        // D16 = 16 bits (1st (y) and 2nd byte (x))
        let i = match opcode.0 {
            0x00 => self.op_00(),                       // NOP
            0x01 => self.op_lxi(Registers::BC, dl, dh), // LXI B,D16
            0x02 => self.op_stax(Registers::BC),        // STAX (BC)
            0x03 => self.op_03(),                       // INX B
            0x04 => self.op_inr(Registers::B),          // INR B
            0x05 => self.op_dcr(Registers::B),          // DCR B
            0x06 => self.op_mvi(Registers::B, dl),      // MVI B, D8
            0x07 => self.op_rotl(false),                // RLC (Rotate left)
            //0x08
            0x09 => self.op_dad(Registers::B),   // DAD BC
            0x0A => self.op_ldax(Registers::BC), // LDAX BC
            0x0B => self.op_dcx(Registers::BC),  // DCX BC
            0x0C => self.op_inr(Registers::C),   // INR C
            //0x0D
            0x0E => self.op_mvi(Registers::C, dl), // MVI C, D8
            0x0F => self.op_rotr(false),           // RRC
            //0x10
            0x11 => self.op_lxi(Registers::DE, dl, dh), // LXI D,D16
            0x12 => self.op_stax(Registers::DE),        // STAX (DE)
            0x13 => self.op_13(),                       // INX D
            0x14 => self.op_inr(Registers::D),          // INR D
            0x15 => self.op_dcr(Registers::D),          // DCR D
            0x16 => self.op_mvi(Registers::D, dl),      // MVI D
            0x17 => self.op_rotl(true),                 // RAL (Rotate left through carry)
            0x19 => self.op_dad(Registers::D),          // DAD D
            0x1A => self.op_ldax(Registers::DE),        // LDAX DE
            0x1B => self.op_dcx(Registers::DE),         // DCX DE
            0x1C => self.op_inr(Registers::E),          // INR E
            0x1E => self.op_mvi(Registers::E, dl),      // MVI E
            0x1F => self.op_rotr(true),                 // RAR
            0x21 => self.op_lxi(Registers::HL, dl, dh), // LXI X,D16
            0x23 => self.op_23(),                       // INX H
            0x24 => self.op_inr(Registers::H),          // INR H
            0x25 => self.op_dcr(Registers::H),          // DCR H
            0x26 => self.op_mvi(Registers::H, dl),      // MVI H, D8
            0x29 => self.op_dad(Registers::H),          // DAD HL
            0x2E => self.op_mvi(Registers::L, dl),      // MVI L
            0x2A => self.lhld(dl, dh),                  // LDA DL DH
            0x2B => self.op_dcx(Registers::HL),         // DCX HL
            0x2C => self.op_inr(Registers::L),          // INR L
            0x31 => self.op_31(dl, dh),                 // LXI SP, D16
            0x32 => self.op_sta(dl, dh),                // STA (adr)<-A
            0x33 => self.op_33(),                       // INX SP
            0x34 => self.op_inr(Registers::HL),         // INR (HL)
            0x35 => self.op_dcr(Registers::HL),         // DCR (HL)
            0x36 => self.op_mvi(Registers::HL, dl),     // MVI (HL)<-D8
            0x3B => self.op_dcx(Registers::SP),         // DCX SP
            0x3C => self.op_inr(Registers::A),          // INR A
            0x3E => self.op_mvi(Registers::A, dl),      // MVI A
            0x40 => self.op_mov(Registers::B, Registers::B), // MOV B <- B
            0x41 => self.op_mov(Registers::B, Registers::C), // MOV B <- C
            0x42 => self.op_mov(Registers::B, Registers::D), // MOV B <- D
            0x43 => self.op_mov(Registers::B, Registers::E), // MOV B <- E
            0x44 => self.op_mov(Registers::B, Registers::H), // MOV B <- H
            0x45 => self.op_mov(Registers::B, Registers::L), // MOV B <- L
            0x46 => self.op_mov(Registers::B, Registers::HL), // MOV B <- (HL)
            0x47 => self.op_mov(Registers::B, Registers::A), // MOV B <- A
            0x6F => self.op_mov(Registers::L, Registers::A), // MOV L <- A
            0x70 => self.op_mov(Registers::HL, Registers::B), // MOV M,B	1		(HL) <- B
            0x71 => self.op_mov(Registers::HL, Registers::C), // MOV M,C	1		(HL) <- C
            0x72 => self.op_mov(Registers::HL, Registers::D), // MOV M,D	1		(HL) <- D
            0x73 => self.op_mov(Registers::HL, Registers::E), // MOV M,E	1		(HL) <- E
            0x74 => self.op_mov(Registers::HL, Registers::H), // MOV M,H	1		(HL) <- H
            0x75 => self.op_mov(Registers::HL, Registers::L), // MOV M,L	1		(HL) <- L
            //0x76 => self.op_76(),              // HLT 1 (special)
            0x77 => self.op_mov(Registers::HL, Registers::A), // MOV M,A
            0x78 => self.op_mov(Registers::A, Registers::B),  // MOV A,B
            0x79 => self.op_mov(Registers::A, Registers::C),  // MOV A,C
            0x7A => self.op_mov(Registers::A, Registers::D),  // MOV A,D
            0x7B => self.op_mov(Registers::A, Registers::E),  // MOV A,E
            0x7C => self.op_mov(Registers::A, Registers::H),  // MOV A,H
            0x7D => self.op_mov(Registers::A, Registers::L),  // MOV A,L
            0x7E => self.op_mov(Registers::A, Registers::HL), // MOV A,(HL)
            0x7F => self.op_mov(Registers::A, Registers::A),  // MOV A,A
            0x80 => self.op_add(Registers::B),                // ADD B
            0x81 => self.op_add(Registers::C),                // ADD C
            0x82 => self.op_add(Registers::D),                // ADD D
            0x83 => self.op_add(Registers::E),                // ADD E
            0x84 => self.op_add(Registers::H),                // ADD H
            0x85 => self.op_add(Registers::L),                // ADD L
            0x86 => self.op_add(Registers::HL),               // ADD M
            0x87 => self.op_add(Registers::A),                // ADD A
            0x88 => self.op_adc(Registers::B),                // ADC B
            0x89 => self.op_adc(Registers::C),                // ADC C
            0x8A => self.op_adc(Registers::D),                // ADC D
            0x8B => self.op_adc(Registers::E),                // ADC E
            0x8C => self.op_adc(Registers::H),                // ADC H
            0x8D => self.op_adc(Registers::L),                // ADC L
            0x8E => self.op_adc(Registers::HL),               // ADC M
            0x8F => self.op_adc(Registers::A),                // ADC A
            0x90 => self.op_sub(Registers::B),                // SUB B
            0x91 => self.op_sub(Registers::C),                // SUB C
            0x92 => self.op_sub(Registers::D),                // SUB D
            0x93 => self.op_sub(Registers::E),                // SUB E
            0x94 => self.op_sub(Registers::H),                // SUB H
            0x95 => self.op_sub(Registers::L),                // SUB L
            0x96 => self.op_sub(Registers::HL),               // SUB M
            0x97 => self.op_sub(Registers::A),                // SUB A
            0xC0 => self.op_rets(super::FLAG_CARRY, false),   // RNC
            0xC1 => self.op_push(Registers::B),               // POP B
            0xC2 => self.op_c2(dl, dh),                       // JNZ
            0xC3 => self.op_c3(dl, dh),                       // JMP
            0xC4 => self.op_call_if(super::FLAG_ZERO, false, dl, dh), // CNZ
            0xC5 => self.op_push(Registers::B),               // PUSH B
            0xC7 => self.op_rst(0b000),                       // RST 0
            0xC8 => self.op_rets(super::FLAG_CARRY, true),    // RC
            0xC9 => self.op_ret(),                            // RET
            0xCC => self.op_call_if(super::FLAG_ZERO, true, dl, dh), // CZ
            0xCF => self.op_rst(0b001),                       // RST 1
            0xD1 => self.op_pop(Registers::D),                // POP D
            0xD3 => self.op_out(dl),                          // OUT
            0xCD => self.op_cd(dl, dh),                       // CALL Addr
            0xD0 => self.op_rets(super::FLAG_CARRY, false),   // RNC
            0xD4 => self.op_call_if(super::FLAG_CARRY, false, dl, dh), // CNC
            0xD5 => self.op_push(Registers::D),               // PUSH D
            0xD7 => self.op_rst(0b010),                       // RST 2
            0xDF => self.op_rst(0b011),                       // RST 3
            0xDC => self.op_call_if(super::FLAG_CARRY, true, dl, dh), // CC
            0xE0 => self.op_rets(super::FLAG_PARITY, false),  // RPO
            0xE1 => self.op_pop(Registers::H),                // POP H
            0xE4 => self.op_call_if(super::FLAG_PARITY, false, dl, dh), // CPO
            0xE5 => self.op_push(Registers::H),               // PUSH H
            0xE7 => self.op_rst(0b100),                       // RST 4
            0xE8 => self.op_rets(super::FLAG_PARITY, true),   // RPE
            0xEB => self.op_xchg(),                           // XCHG
            0xEC => self.op_call_if(super::FLAG_PARITY, true, dl, dh), // CPE
            0xEF => self.op_rst(0b101),                       // RST 5
            0xF0 => self.op_rets(super::FLAG_SIGN, false),    // RP
            0xF1 => self.op_pop(Registers::PSW),              // POP PSW
            0xF4 => self.op_call_if(super::FLAG_SIGN, false, dl, dh), // CP
            0xF5 => self.op_push(Registers::PSW),             // Push PSW
            0xFE => self.op_fe(dl),                           // CPI
            0xF7 => self.op_rst(0b110),                       // RST 6
            0xF8 => self.op_rets(super::FLAG_SIGN, true),     // RM
            0xFC => self.op_call_if(super::FLAG_SIGN, true, dl, dh), // CM
            0xFF => self.op_rst(0b111),                       // RST 7
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown!!",
                    opcode.0, opcode.0
                ))
            }
        };

        match i {
            ProgramCounter::Next => self.pc += super::OPCODE_SIZE,
            ProgramCounter::Two => self.pc += super::OPCODE_SIZE * 2,
            ProgramCounter::Three => self.pc += super::OPCODE_SIZE * 3,
            ProgramCounter::Jump(d) => self.pc = d,
        }

        Ok(())
    }

    // LHLD
    pub fn lhld(&mut self, dl: u8, dh: u8) -> ProgramCounter {
        let mut addr: u16 = u16::from(dh) << 8 | u16::from(dl);
        self.l = match self.memory.get(addr as usize) {
            Some(&v) => v,
            None => 0,
        };
        addr = addr.overflowing_add(0x01).0;
        self.h = match self.memory.get(addr as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Three
    }

    // OUT D8
    // Would send the contents of accumulator to the device sent
    // as the data portion of this command
    pub fn op_out(&self, _data: u8) -> ProgramCounter {
        ProgramCounter::Two
    }

    pub fn op_00(&self) -> ProgramCounter {
        ProgramCounter::Next
    }

    // INX B
    pub fn op_03(&mut self) -> ProgramCounter {
        let mut bc_pair: u16 = self.get_register_pair(Registers::BC);
        bc_pair = bc_pair.overflowing_add(0x01).0; // overflowing_add returns (v, t/f for overflow);

        self.set_register_pair(Registers::BC, bc_pair);

        ProgramCounter::Next
    }

    // Store accumulator direct to location in memory specified
    // by address dhdl
    pub fn op_sta(&mut self, dl: u8, dh: u8) -> ProgramCounter {
        let addr: u16 = u16::from(dh) << 8 | u16::from(dl);
        self.memory[addr as usize] = self.a;

        ProgramCounter::Three
    }

    // Returns true if an addition will case an aux carry
    pub fn will_ac(&mut self, a: u8, b: u8) -> bool {
        ((a & 0x0F) + (b & 0x0F)) & 0x10 == 0x10
    }

    // INR Reg
    // Flags affected: Z,S,P,AC
    pub fn op_inr(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                let (res, of) = self.b.overflowing_add(1);
                let ac = self.will_ac(1, self.b);
                self.update_flags(res, of, ac);
                self.b = res;
            }
            Registers::C => {
                let (res, of) = self.c.overflowing_add(1);
                let ac = self.will_ac(1, self.c);
                self.update_flags(res, of, ac);
                self.c = res;
            }
            Registers::D => {
                let (res, of) = self.d.overflowing_add(1);
                let ac = self.will_ac(1, self.d);
                self.update_flags(res, of, ac);
                self.d = res;
            }
            Registers::E => {
                let (res, of) = self.e.overflowing_add(1);
                let ac = self.will_ac(1, self.d);
                self.update_flags(res, of, ac);
                self.e = res;
            }
            Registers::H => {
                let (res, of) = self.h.overflowing_add(1);
                let ac = self.will_ac(1, self.h);
                self.update_flags(res, of, ac);
                self.h = res;
            }
            Registers::L => {
                let (res, of) = self.l.overflowing_add(1);
                let ac = self.will_ac(1, self.l);
                self.update_flags(res, of, ac);
                self.l = res;
            }
            Registers::HL => {
                let val = self.memory[self.get_addr_pointer()];
                let ac = self.will_ac(1, val);
                let (res, of) = val.overflowing_add(1);
                self.update_flags(res, of, ac);
                self.memory[self.get_addr_pointer()] = res;
            }
            Registers::A => {
                let (res, of) = self.a.overflowing_add(1);
                let ac = self.will_ac(1, self.a);
                self.update_flags(res, of, ac);
                self.a = res;
            }
            _ => (),
        }

        ProgramCounter::Next
    }

    // SUB A (Subtract register param from A)
    // Flags affected: Z, S, P, CY, AC
    pub fn op_sub(&mut self, reg: Registers) -> ProgramCounter {
        let o: (u8, bool) = match reg {
            Registers::A => self.a.overflowing_sub(self.a),
            Registers::B => self.a.overflowing_sub(self.b),
            Registers::C => self.a.overflowing_sub(self.c),
            Registers::D => self.a.overflowing_sub(self.d),
            Registers::E => self.a.overflowing_sub(self.e),
            Registers::H => self.a.overflowing_sub(self.h),
            Registers::L => self.a.overflowing_sub(self.l),
            Registers::HL => self.a.overflowing_sub(self.memory[self.get_addr_pointer()]),
            _ => (0_u8, false),
        };

        self.update_flags(o.0, o.1, (1 & 0x0F) > (self.a & 0x0F));
        self.a = o.0;
        ProgramCounter::Next
    }

    // DCR Reg
    // Flags affected: Z,S,P,AC
    pub fn op_dcr(&mut self, reg: Registers) -> ProgramCounter {
        //let new_val = self.b.wrapping_sub(1);

        match reg {
            Registers::B => {
                let (res, of) = self.b.overflowing_sub(1);
                self.update_flags(res, of, (1 & 0x0F) > (self.b & 0x0F));
                self.b = res;
            }
            Registers::D => {
                let (res, of) = self.d.overflowing_sub(1);
                self.update_flags(res, of, (1 & 0x0F) > (self.d & 0x0F));
                self.d = res;
            }
            Registers::H => {
                let (res, of) = self.h.overflowing_sub(1);
                self.update_flags(res, of, (1 & 0x0F) > (self.h & 0x0F));
                self.h = res;
            }
            Registers::HL => {
                let mem = self.memory[self.get_addr_pointer()];
                let (res, of) = mem.overflowing_sub(1);
                self.update_flags(res, of, (1 & 0x0F) > (mem & 0x0F));
                self.memory[self.get_addr_pointer()] = res;
            }

            _ => (),
        }

        ProgramCounter::Next
    }

    // Exchanges registers DE with HL
    pub fn op_xchg(&mut self) -> ProgramCounter {
        let d = self.d;
        let e = self.e;

        self.d = self.h;
        self.e = self.l;
        self.h = d;
        self.l = e;

        ProgramCounter::Next
    }

    // Pushes onto stack according to the register pair requested
    // (sp-2)<-P2; (sp-1)<-P1; sp <- sp - 2
    pub fn op_push(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                // BC Pair 0xC5
                self.memory[usize::from(self.sp - 2)] = self.c;
                self.memory[usize::from(self.sp - 1)] = self.b;
            }
            Registers::D => {
                // DE Pair 0xD5
                self.memory[usize::from(self.sp - 2)] = self.e;
                self.memory[usize::from(self.sp - 1)] = self.d;
            }
            Registers::H => {
                // HL Pair 0xE5
                self.memory[usize::from(self.sp - 2)] = self.l;
                self.memory[usize::from(self.sp - 1)] = self.h;
            }
            Registers::PSW => {
                // PSW 0xF5
                self.memory[usize::from(self.sp - 2)] = self.flags;
                self.memory[usize::from(self.sp - 1)] = self.a;
            }
            _ => (),
        };
        self.sp -= 2;
        ProgramCounter::Next
    }

    // Pops from the stack according to the register pair requested
    // 	L <- (sp); H <- (sp+1); sp <- sp+2
    pub fn op_pop(&mut self, reg: Registers) -> ProgramCounter {
        match reg {
            Registers::B => {
                // BC Pair 0xC1
                self.c = self.memory[usize::from(self.sp)];
                self.b = self.memory[usize::from(self.sp + 1)];
            }
            Registers::D => {
                // DE Pair 0xD1
                self.e = self.memory[usize::from(self.sp)];
                self.d = self.memory[usize::from(self.sp + 1)];
            }
            Registers::H => {
                // HL Pair 0xE1
                self.l = self.memory[usize::from(self.sp)];
                self.h = self.memory[usize::from(self.sp + 1)];
            }
            Registers::PSW => {
                // PSW 0xF1
                self.flags = self.memory[usize::from(self.sp)];
                self.a = self.memory[usize::from(self.sp + 1)];
            }
            _ => (),
        };
        self.sp += 2;

        ProgramCounter::Next
    }

    // Performs the MVI functionality
    pub fn op_mvi(&mut self, target: Registers, x: u8) -> ProgramCounter {
        match target {
            Registers::A => self.a = x,                                // 0x3E
            Registers::B => self.b = x,                                // 0x06
            Registers::C => self.c = x,                                // 0x0E
            Registers::D => self.d = x,                                // 0x16
            Registers::E => self.e = x,                                // 0x1E
            Registers::H => self.h = x,                                // 0x26
            Registers::L => self.l = x,                                // 0x2E
            Registers::HL => self.memory[self.get_addr_pointer()] = x, // 0x36
            _ => (),                                                   // Do nothing
        };
        ProgramCounter::Two
    }

    // Performs the Double Add (DAD) functionality
    // Sets H to the value according to the supplied register
    // Basically: HL = HL+<Selected register pair>
    pub fn op_dad(&mut self, source: Registers) -> ProgramCounter {
        //let val = usize::from(u16::from(self.h) << 8 | u16::from(self.l));
        let val = usize::from(self.get_register_pair(Registers::HL));

        let src: usize = match source {
            Registers::B | Registers::BC => usize::from(self.get_register_pair(Registers::BC)),
            Registers::D | Registers::DE => usize::from(self.get_register_pair(Registers::DE)),
            Registers::H | Registers::HL => val,
            _ => 0,
        };

        let (new, of) = val.overflowing_add(src);

        self.h = (new >> 8) as u8;
        self.l = (new & 0xFF) as u8;

        if of {
            self.set_flag(super::FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // LXI (target pair), D16
    pub fn op_lxi(&mut self, target: Registers, x: u8, y: u8) -> ProgramCounter {
        match target {
            Registers::BC => {
                self.b = y;
                self.e = x;
            }
            Registers::DE => {
                self.d = y;
                self.e = x;
            }
            Registers::HL => {
                self.h = y;
                self.l = x;
            }
            _ => (),
        }
        ProgramCounter::Three
    }

    // INX D
    pub fn op_13(&mut self) -> ProgramCounter {
        let mut c: u16 = u16::from(self.d) << 8 | u16::from(self.e);
        c = c.overflowing_add(0x01).0; // overflowing_add returns (v, t/f for overflow);
        self.d = (c >> 8) as u8;
        self.e = (c & 0xFF) as u8;

        ProgramCounter::Next
    }

    // LDAX
    // Loads A with value from memory location specified by register pair
    pub fn op_ldax(&mut self, reg: Registers) -> ProgramCounter {
        let loc: u16 = match reg {
            Registers::DE => u16::from(self.d) << 8 | u16::from(self.e),
            Registers::BC => u16::from(self.b) << 8 | u16::from(self.c),
            _ => {
                return ProgramCounter::Next;
            }
        };

        self.a = match self.memory.get(loc as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Next
    }

    // DCX
    pub fn op_dcx(&mut self, reg: Registers) -> ProgramCounter {
        let mut val = self.get_register_pair(reg);
        val = val.overflowing_sub(1).0;
        self.set_register_pair(reg, val);

        ProgramCounter::Next
    }

    // INX H
    pub fn op_23(&mut self) -> ProgramCounter {
        let mut c: u16 = u16::from(self.h) << 8 | u16::from(self.l);
        c = c.overflowing_add(0x01).0; // overflowing_add returns (v, t/f for overflow);
        self.h = (c >> 8) as u8;
        self.l = (c & 0xFF) as u8;

        ProgramCounter::Next
    }

    // Load Stack Pointer with the value (y<<8|x)
    // SP.hi <- byte 3, SP.lo <- byte 2
    pub fn op_31(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.sp = u16::from(y) << 8 | u16::from(x);
        ProgramCounter::Three
    }

    // INX SP
    pub fn op_33(&mut self) -> ProgramCounter {
        self.sp = self.sp.overflowing_add(0x01).0; // overflowing_add returns (v, t/f for overflow);

        ProgramCounter::Next
    }

    // MOV T(arget), Registers::X
    // Moves into T(arget) the value in register specified by the enum Registers
    fn op_mov(&mut self, target: Registers, source: Registers) -> ProgramCounter {
        let val = match source {
            Registers::A => self.a,
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::L => self.l,
            Registers::H => self.h,
            Registers::HL => self.memory[self.get_addr_pointer()],
            _ => self.l, // Ignored
        };

        match target {
            Registers::A => self.a = val,
            Registers::B => self.b = val,
            Registers::C => self.c = val,
            Registers::D => self.d = val,
            Registers::E => self.e = val,
            Registers::L => self.l = val,
            Registers::H => self.l = val,
            Registers::HL => self.memory[self.get_addr_pointer()] = val,
            _ => (), // Do nothing
        };

        ProgramCounter::Next
    }

    // The contents of the program counter (16bit)
    // are pushed onto the stack, providing a return address for
    // later use by a RETURN instruction.
    // Program execution continues at memory address:
    // OOOOOOOOOOEXPOOOB
    pub fn op_rst(&mut self, loc: u8) -> ProgramCounter {
        self.memory[usize::from(self.sp - 2)] = (self.pc as u16 >> 8) as u8;
        self.memory[usize::from(self.sp - 1)] = (self.pc as u16 & 0xFF) as u8;
        self.sp -= 2;

        ProgramCounter::Jump((loc << 3) as usize)
    }

    // JNZ (Jump if nonzero)
    pub fn op_c2(&mut self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        match self.test_flag(super::FLAG_ZERO) {
            true => ProgramCounter::Three,
            false => ProgramCounter::Jump(dest.into()),
        }
    }

    // Jump to a given location as provided by (y<<8 | x)
    pub fn op_c3(&self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        ProgramCounter::Jump(dest.into())
    }

    // RET (PC.lo <- (sp); PC.hi<-(sp+1); SP <- SP+2)
    pub fn op_ret(&mut self) -> ProgramCounter {
        let pc_lo = match self.memory.get(usize::from(self.sp)) {
            Some(&v) => v,
            None => 0,
        };
        let pc_hi = match self.memory.get(usize::from(self.sp + 1)) {
            Some(&v) => v,
            None => 0,
        };
        let dest: u16 = u16::from(pc_hi) << 8 | u16::from(pc_lo);
        self.pc = dest as usize; // Set our PC back to where we were
        self.sp += 2;

        //ProgramCounter::Jump(dest.into())
        ProgramCounter::Three // And go to the next op
    }

    // Returns if the flag supplied's value matches the supplied sign
    pub fn op_rets(&mut self, flag: u8, sign: bool) -> ProgramCounter {
        if sign == self.test_flag(flag) {
            return self.op_ret();
        }

        ProgramCounter::Next
    }

    // Calls if the flag's supplied value matches the supplied sign
    pub fn op_call_if(&mut self, flag: u8, sign: bool, x: u8, y: u8) -> ProgramCounter {
        if sign == self.test_flag(flag) {
            return self.op_cd(x, y);
        }
        ProgramCounter::Three
    }

    // (SP-1)<-PC.hi;(SP-2)<-PC.lo;SP<-SP-2;PC=adr
    pub fn op_cd(&mut self, x: u8, y: u8) -> ProgramCounter {
        // Save away the current PC hi/lo into the stack
        let pc_hi = self.pc >> 8;
        let pc_lo = self.pc & 0xFF;
        self.memory[usize::from(self.sp - 1)] = pc_hi as u8;
        self.memory[usize::from(self.sp - 2)] = pc_lo as u8;
        self.sp -= 2;

        // Tell the program counter where we want to go next
        let ys: u16 = u16::from(y) << 8;
        self.pc = usize::from(ys | u16::from(x));

        ProgramCounter::Jump(self.pc)
    }

    // CPI - Compare D16 to Accum, set flags accordingly
    pub fn op_fe(&mut self, data: u8) -> ProgramCounter {
        // Subtract the data from register A and set flags on the result
        let (res, overflow) = self.a.overflowing_sub(data);
        let aux_carry = (self.a & 0x0F).wrapping_sub(data & 0x0F) > 0x0F;

        self.update_flags(res, overflow, aux_carry);

        ProgramCounter::Two
    }

    // Rotates left, if through_carry is true, it does that.
    pub fn op_rotl(&mut self, through_carry: bool) -> ProgramCounter {
        // Store off our current carry bit
        let carry_bit = self.test_flag(super::FLAG_CARRY);

        // Store off our current high order bit
        let high_order = self.a >> 7;

        // Shift one position to the left
        let mut new_accum: u8 = self.a << 1;

        match through_carry {
            true => {
                new_accum |= carry_bit as u8; // carry bit replaces low order
            }
            // Normal carry
            false => {
                new_accum |= high_order as u8; // high order replaces low order
            }
        };

        self.a = new_accum;

        // Update the carry bit with the old high order bit
        if high_order > 0 {
            self.set_flag(super::FLAG_CARRY);
        } else {
            self.reset_flag(super::FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // Rotates right, if through_carry is true, it does that.
    pub fn op_rotr(&mut self, through_carry: bool) -> ProgramCounter {
        // Store off our current carry bit
        let carry_bit = self.test_flag(super::FLAG_CARRY);
        let low_order = self.a & 0x01; // Save off the low order bit so we can rotate it.

        let mut new_accum: u8 = self.a >> 1;

        match through_carry {
            true => {
                new_accum |= (carry_bit as u8) << 7; // Carry bit replaces high order
            }
            // Normal carry
            false => {
                new_accum |= low_order << 7; // Low order replaces high order
            }
        };

        self.a = new_accum;

        if low_order > 0 {
            self.set_flag(super::FLAG_CARRY);
        } else {
            self.reset_flag(super::FLAG_CARRY);
        }

        ProgramCounter::Next
    }

    // Add to the accumulator the supplied register
    // as well as update flags
    pub fn op_add(&mut self, reg: Registers) -> ProgramCounter {
        let to_add: u8 = match reg {
            Registers::B => self.b,
            Registers::C => self.c,
            Registers::D => self.d,
            Registers::E => self.e,
            Registers::H => self.h,
            Registers::L => self.l,
            Registers::HL => self.memory[self.get_addr_pointer()],
            Registers::A => self.a,
            _ => 0_u8,
        };

        let (res, of) = self.a.overflowing_add(to_add);
        let ac = self.will_ac(to_add, self.a);
        self.a = res;
        self.update_flags(res, of, ac);

        ProgramCounter::Next
    }

    // Add to the accumulator the supplied register
    // along with the CARRY flag's value
    // as well as update flags
    pub fn op_adc(&mut self, reg: Registers) -> ProgramCounter {
        let to_add: u8 = self.test_flag(super::FLAG_CARRY) as u8
            + match reg {
                Registers::B => self.b,
                Registers::C => self.c,
                Registers::D => self.d,
                Registers::E => self.e,
                Registers::H => self.h,
                Registers::L => self.l,
                Registers::HL => self.memory[self.get_addr_pointer()],
                Registers::A => self.a,
                _ => 0_u8,
            };

        let (res, of) = self.a.overflowing_add(to_add);
        let ac = self.will_ac(to_add, self.a);
        self.a = res;
        self.update_flags(res, of, ac);

        ProgramCounter::Next
    }

    // Stores accumulator at memory location of supplied register
    pub fn op_stax(&mut self, reg: Registers) -> ProgramCounter {
        // Get our location first
        let location = match reg {
            Registers::BC => Some(self.get_register_pair(Registers::BC)),
            Registers::DE => Some(self.get_register_pair(Registers::DE)),
            _ => None,
        };

        // Update memory with the value of the accumulator
        if let Some(l) = location {
            self.memory[l as usize] = self.a
        }

        ProgramCounter::Next
    }
}
