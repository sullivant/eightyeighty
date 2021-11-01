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
    BC, // A register pair
    DE, // A register pair
    HL, // A register pair, used to reference memory locations
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
            _ => 0 as u16,
        }
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

        if overflow {
            self.set_flag(super::FLAG_CARRY);
        }

        if aux_carry {
            self.set_flag(super::FLAG_AUXCARRY);
        }
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
            0x00 => self.op_00(),                             // NOP
            0x03 => self.op_03(),                             // INX B
            0x05 => self.op_05(),                             // DCR B
            0x06 => self.op_mvi(Registers::B, dl),            // MVI B, D8
            0x09 => self.op_dad(Registers::B),                // DAD BC
            0x0E => self.op_mvi(Registers::C, dl),            // MVI C, D8
            0x11 => self.op_11(dl, dh),                       // LXI D,D16
            0x13 => self.op_13(),                             // INX D
            0x16 => self.op_mvi(Registers::D, dl),            // MVI D
            0x1A => self.op_1a(),                             // LDAX D
            0x1E => self.op_mvi(Registers::E, dl),            // MVI E
            0x21 => self.op_21(dl, dh),                       // LXI X,D16
            0x23 => self.op_23(),                             // INX H
            0x26 => self.op_mvi(Registers::H, dl),            // MVI H, D8
            0x29 => self.op_dad(Registers::H),                // DAD HL
            0x2E => self.op_mvi(Registers::L, dl),            // MVI L
            0x31 => self.op_31(dl, dh),                       // LXI SP, D16
            0x33 => self.op_33(),                             // INX SP
            0x36 => self.op_mvi(Registers::HL, dl),           // MVI (HL)<-D8
            0x3E => self.op_mvi(Registers::A, dl),            // MVI A
            0x6F => self.op_mov(Registers::L, Registers::A),  // MOV L <- A
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
            0xC2 => self.op_c2(dl, dh),                       // JNZ
            0xC3 => self.op_c3(dl, dh),                       // JMP
            0xC5 => self.op_push(Registers::B),               // PUSH B
            0xC9 => self.op_c9(),                             // RET
            0xCD => self.op_cd(dl, dh),                       // CALL Addr
            0xD5 => self.op_push(Registers::D),               // PUSH D
            0xF4 => self.op_f4(dl, dh),                       // CP If Plus
            0xFE => self.op_fe(dl),                           // CPI
            0xE5 => self.op_push(Registers::H),               // PUSH H
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

    pub fn op_00(&self) -> ProgramCounter {
        ProgramCounter::Next
    }

    // INX B
    pub fn op_03(&mut self) -> ProgramCounter {
        let mut bc_pair: u16 = self.get_register_pair(Registers::BC);
        bc_pair = bc_pair.overflowing_add(0x01).0; // overflowing_add returns (v, t/f for overflow);
        self.b = (bc_pair >> 8) as u8;
        self.c = (bc_pair & 0xFF) as u8;

        ProgramCounter::Next
    }

    // DCR B
    // Flags affected: Z,S,P,AC
    pub fn op_05(&mut self) -> ProgramCounter {
        //let new_val = self.b.wrapping_sub(1);
        let (res, of) = self.b.overflowing_sub(1);
        let ac = (1 & 0x0F) > (self.b & 0x0F);

        self.update_flags(res, of, ac);
        self.b = res;

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
            _ => (),
        };
        self.sp -= 2;
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
            Registers::B => usize::from(self.get_register_pair(Registers::BC)),
            Registers::H => val,
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

    // LXI D, D16
    pub fn op_11(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.d = y;
        self.e = x;
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

    // LDAX DE (A <- DE)
    pub fn op_1a(&mut self) -> ProgramCounter {
        let loc: u16 = u16::from(self.d) << 8 | u16::from(self.e);

        self.a = match self.memory.get(loc as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Next
    }

    // LXI H,D16
    pub fn op_21(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.h = y;
        self.l = x;
        ProgramCounter::Three
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
    pub fn op_c9(&mut self) -> ProgramCounter {
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

    // Call if Plus
    pub fn op_f4(&mut self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);

        // If FLAG_SIGN is zero the result was positive
        // so we call (jump) to our destination
        match self.test_flag(super::FLAG_SIGN) {
            true => ProgramCounter::Three,
            false => ProgramCounter::Jump(dest.into()),
        }
    }

    // CPI - Compare D16 to Accum, set flags accordingly
    pub fn op_fe(&mut self, data: u8) -> ProgramCounter {
        // Subtract the data from register A and set flags on the result
        let (res, overflow) = self.a.overflowing_sub(data);
        let aux_carry = (self.a & 0x0F).wrapping_sub(data & 0x0F) > 0x0F;

        self.update_flags(res, overflow, aux_carry);

        ProgramCounter::Two
    }
}
