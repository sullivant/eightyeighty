use std::fmt;

pub(crate) mod instructions;
mod tests;

use crate::{
    bus::Bus, constants::{FLAG_AUXCARRY, FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO}
};
use instructions::Instruction;

#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct CPU {
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

    halted: bool, 

    interrupts_enabled: bool,

    pub cycle_count: usize,                 // Cycle count
    pub current_instruction: Instruction,   // Used in cpu.run_opcode()
    pub next_instruction: Instruction,      // Populated after run_opcode() but before next tick()
}

/// Will describe the output of a single tick's step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub pc: usize,
    pub opcode: u8,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub cycles: u8,
    pub registers: RegistersSnapshot,
}


impl fmt::Display for StepResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PC: 0x{:04X}  Opcode: 0x{:02X}  Mnemonic: {}", self.pc, self.opcode, self.mnemonic)?;
        write!(f, "Bytes:")?;
        for b in &self.bytes {
            write!(f, " {b:02X}")?;
        }
        writeln!(f)?;
        writeln!(f, "Cycles: {}", self.cycles)?;
        writeln!(f, "{}", self.registers)
    }
}

#[derive(Debug, Clone)]
pub struct RegistersSnapshot {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: u8,
}

impl fmt::Display for RegistersSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Registers:")?;
        writeln!(f, "  A:  0x{:02X}  B: 0x{:02X}  C: 0x{:02X}", self.a, self.b, self.c)?;
        writeln!(f, "  D:  0x{:02X}  E: 0x{:02X}  H: 0x{:02X}", self.d, self.e, self.h)?;
        writeln!(f, "  L:  0x{:02X}  SP: 0x{:04X}  PC: 0x{:04X}", self.l, self.sp, self.pc)?;
        writeln!(f, "  Flags: 0x{:02X}", self.flags)
    }
}

#[allow(unused)]
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
    SP, // Stack pointer
    SW, // Program Status Word
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
            Registers::SW => write!(f, "SW"),
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    #[must_use]
    pub const fn new() -> CPU {
        CPU {
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

            halted: false,
            interrupts_enabled: true,

            cycle_count: 1,
            current_instruction: Instruction::new(0x00), 
            next_instruction: Instruction::new(0x00) 
        }
    }

    /// Performs a basic CPU reset without the need to re-create the entire CPU
    pub fn reset(&mut self) -> Result<(), String> {
        self.pc = 0x00;
        self.sp = 0x00;
        self.a = 0x00;
        self.b = 0x00;
        self.c = 0x00;
        self.d = 0x00;
        self.e = 0x00;
        self.h = 0x00;
        self.l = 0x00;
        self.flags = 0x02;
        self.halted = false;
        self.cycle_count = 1;
        self.current_instruction = Instruction::new(0x00);
        self.next_instruction = Instruction::new(0x00);

        Ok(())
    }

    // Reads an instruction at ProgramCounter
    pub fn read_instruction(&mut self, bus: &Bus) -> Instruction {
        let opcode = bus.read(self.pc);

        Instruction::new(opcode) // new() will fill in the rest..
    }

    pub fn step(&mut self, bus: &mut Bus) -> Result<StepResult , String> {
        let pc_start = self.pc; // Where we are starting from

        // Fetch opcode Instruction and set it to "current"
        let opcode = self.read_instruction(bus); // Gather the current opcode to run, based on PC's location
        self.current_instruction = opcode;

        // Capture what this instruction is, so we can debug with StepResult
        let mut bytes = Vec::with_capacity(opcode.size);
        for i in 0..opcode.size {
            bytes.push (
                bus.read(self.pc + i),
            );
        }

        // Execute the opcode
        let cycles_ran = self.run_opcode(bus)?;

        // Snapshot of the registers after execution
        let registers = RegistersSnapshot {
            a: self.a,
            b: self.b,
            c: self.c,
            d: self.d,
            e: self.e,
            h: self.h,
            l: self.l,
            sp: self.sp,
            pc: self.pc as u16,
            flags: self.flags,
        };

        Ok(StepResult {
            pc: pc_start,
            opcode: opcode.opcode,
            bytes,
            mnemonic: opcode.text.to_string(),
            cycles: cycles_ran,
            registers
        })

    }

    // Gathers the data necessary for the instruction and
    // calls out to the appropriate instruction operation to
    // perform the thing...
    #[allow(clippy::too_many_lines)]
    pub fn run_opcode(&mut self, bus: &mut Bus) -> Result<u8, String> {
        // let (dl, dh) = match self.get_data_pair() {
        //     Ok(value) => value,
        //     Err(_) => return Err("Unable to get data pair".to_string()),
        // };
        let (dl, dh) = self.get_data_pair(bus);

        // Used in determining if PC actually changed in the opcode, like in a jump, etc.
        let pc_before = self.pc;

        // Returned from each opcode.  Sometimes modified in the opcode operation, so that may
        // be overridden in a particular opcode such as jump, etc.
        let code_cycles = self.current_instruction.cycles;

        // Do the actual run of the opcode and return the result
        let opcode_result: Result<u8, String> = match self.current_instruction.opcode {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Ok(code_cycles),

            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => self.mvi(dl, bus),

            0x09 | 0x19 | 0x29 | 0x39 => self.dad(),

            0x01 => self.lxi(Registers::BC, dl, dh),
            0x02 => self.stax(Registers::BC, bus), // STAX (BC)
            0x03 => self.inx(Registers::BC),
            0x04 => self.inr(Registers::B, bus),
            0x05 => self.dcr(Registers::B, bus),
            0x07 => self.rlc_ral(false),
            0x0A => self.ldax(Registers::BC, bus),
            0x0B => self.dcx(Registers::BC),
            0x0C => self.inr(Registers::C, bus),
            0x0D => self.dcr(Registers::C, bus),
            0x0F => self.rrc_rar(true),

            0x11 => self.lxi(Registers::DE, dl, dh),
            0x12 => self.stax(Registers::DE, bus), // STAX (DE)
            0x13 => self.inx(Registers::DE),
            0x14 => self.inr(Registers::D, bus),
            0x15 => self.dcr(Registers::D, bus),
            0x17 => self.rlc_ral(true),
            0x1A => self.ldax(Registers::DE, bus),
            0x1B => self.dcx(Registers::DE),
            0x1C => self.inr(Registers::E, bus),
            0x1D => self.dcr(Registers::E, bus),
            0x1F => self.rrc_rar(false),

            0x21 => self.lxi(Registers::HL, dl, dh),
            0x22 => self.shld(dl, dh, bus),
            0x2A => self.lhld(dl, dh, bus),
            0x23 => self.inx(Registers::HL),
            0x24 => self.inr(Registers::H, bus),
            0x25 => self.dcr(Registers::H, bus),
            0x27 => self.daa(),
            0x2B => self.dcx(Registers::HL),
            0x2C => self.inr(Registers::L, bus),
            0x2D => self.dcr(Registers::L, bus),
            0x2F => {
                self.a = !self.a; // Complement the accumulator
                Ok(code_cycles)
            }

            0x31 => self.lxi(Registers::SP, dl, dh),
            0x32 => self.sta(dl, dh, bus), // STA (adr)<-A
            0x33 => self.inx(Registers::SP),
            0x34 => self.inr(Registers::HL, bus),
            0x35 => self.dcr(Registers::HL, bus),
            0x37 => {
                self.set_flag(FLAG_CARRY);
                Ok(code_cycles)
            }
            0x3A => self.lda(dl, dh, bus),
            0x3B => self.dcx(Registers::SP),
            0x3C => self.inr(Registers::A, bus),
            0x3D => self.dcr(Registers::A, bus),
            0x3F => {
                // Complement the carry flag
                if self.test_flag(FLAG_CARRY) {
                    self.reset_flag(FLAG_CARRY);
                } else {
                    self.set_flag(FLAG_CARRY);
                }
                Ok(code_cycles)
            }

            0x40 => self.mov(Registers::B, Registers::B, bus), // MOV B <- B
            0x41 => self.mov(Registers::B, Registers::C, bus), // MOV B <- C
            0x42 => self.mov(Registers::B, Registers::D, bus), // MOV B <- D
            0x43 => self.mov(Registers::B, Registers::E, bus), // MOV B <- E
            0x44 => self.mov(Registers::B, Registers::H, bus), // MOV B <- H
            0x45 => self.mov(Registers::B, Registers::L, bus), // MOV B <- L
            0x46 => self.mov(Registers::B, Registers::HL, bus), // MOV B <- (HL)
            0x47 => self.mov(Registers::B, Registers::A, bus), // MOV B <- A
            0x48 => self.mov(Registers::C, Registers::B, bus), // MOV C <- B
            0x49 => self.mov(Registers::C, Registers::C, bus), // MOV C <- C
            0x4A => self.mov(Registers::C, Registers::D, bus), // MOV C <- D
            0x4B => self.mov(Registers::C, Registers::E, bus), // MOV C <- E
            0x4C => self.mov(Registers::C, Registers::H, bus), // MOV C <- H
            0x4D => self.mov(Registers::C, Registers::L, bus), // MOV C <- L
            0x4E => self.mov(Registers::C, Registers::HL, bus), // MOV C <- HL
            0x4F => self.mov(Registers::C, Registers::A, bus), // MOV C <- A

            0x50 => self.mov(Registers::D, Registers::B, bus), // MOV D <- B
            0x51 => self.mov(Registers::D, Registers::C, bus), // MOV D <- C
            0x52 => self.mov(Registers::D, Registers::D, bus), // MOV D <- D
            0x53 => self.mov(Registers::D, Registers::E, bus), // MOV D <- E
            0x54 => self.mov(Registers::D, Registers::H, bus), // MOV D <- H
            0x55 => self.mov(Registers::D, Registers::L, bus), // MOV D <- L
            0x56 => self.mov(Registers::D, Registers::HL, bus), // MOV D <- (HL)
            0x57 => self.mov(Registers::D, Registers::A, bus), // MOV D <- A
            0x58 => self.mov(Registers::E, Registers::B, bus), // MOV E <- B
            0x59 => self.mov(Registers::E, Registers::C, bus), // MOV E <- C
            0x5A => self.mov(Registers::E, Registers::D, bus), // MOV E <- D
            0x5B => self.mov(Registers::E, Registers::E, bus), // MOV E <- E
            0x5C => self.mov(Registers::E, Registers::H, bus), // MOV E <- H
            0x5D => self.mov(Registers::E, Registers::L, bus), // MOV E <- L
            0x5E => self.mov(Registers::E, Registers::HL, bus), // MOV E <- HL
            0x5F => self.mov(Registers::E, Registers::A, bus), // MOV E <- A

            0x60 => self.mov(Registers::H, Registers::B, bus), // MOV H <- B
            0x61 => self.mov(Registers::H, Registers::C, bus), // MOV H <- C
            0x62 => self.mov(Registers::H, Registers::D, bus), // MOV H <- D
            0x63 => self.mov(Registers::H, Registers::E, bus), // MOV H <- E
            0x64 => self.mov(Registers::H, Registers::H, bus), // MOV H <- H
            0x65 => self.mov(Registers::H, Registers::L, bus), // MOV H <- L
            0x66 => self.mov(Registers::H, Registers::HL, bus), // MOV H <- (HL)
            0x67 => self.mov(Registers::H, Registers::A, bus), // MOV H <- A
            0x68 => self.mov(Registers::L, Registers::B, bus), // MOV L <- B
            0x69 => self.mov(Registers::L, Registers::C, bus), // MOV L <- C
            0x6A => self.mov(Registers::L, Registers::D, bus), // MOV L <- D
            0x6B => self.mov(Registers::L, Registers::E, bus), // MOV L <- E
            0x6C => self.mov(Registers::L, Registers::H, bus), // MOV L <- H
            0x6D => self.mov(Registers::L, Registers::L, bus), // MOV L <- L
            0x6E => self.mov(Registers::L, Registers::HL, bus), // MOV L <- HL
            0x6F => self.mov(Registers::L, Registers::A, bus), // MOV L <- A

            0x70 => self.mov(Registers::HL, Registers::B, bus), // MOV M,B	1		(HL) <- B
            0x71 => self.mov(Registers::HL, Registers::C, bus), // MOV M,C	1		(HL) <- C
            0x72 => self.mov(Registers::HL, Registers::D, bus), // MOV M,D	1		(HL) <- D
            0x73 => self.mov(Registers::HL, Registers::E, bus), // MOV M,E	1		(HL) <- E
            0x74 => self.mov(Registers::HL, Registers::H, bus), // MOV M,H	1		(HL) <- H
            0x75 => self.mov(Registers::HL, Registers::L, bus), // MOV M,L	1		(HL) <- L
            0x76 => self.hlt(),
            0x77 => self.mov(Registers::HL, Registers::A, bus), // MOV M,A
            0x78 => self.mov(Registers::A, Registers::B, bus),  // MOV A,B
            0x79 => self.mov(Registers::A, Registers::C, bus),  // MOV A,C
            0x7A => self.mov(Registers::A, Registers::D, bus),  // MOV A,D
            0x7B => self.mov(Registers::A, Registers::E, bus),  // MOV A,E
            0x7C => self.mov(Registers::A, Registers::H, bus),  // MOV A,H
            0x7D => self.mov(Registers::A, Registers::L, bus),  // MOV A,L
            0x7E => self.mov(Registers::A, Registers::HL, bus), // MOV A,(HL)
            0x7F => self.mov(Registers::A, Registers::A, bus),  // MOV A,A

            0x80..=0x87 => self.add(bus),
            0x88..=0x8F => self.adc(bus),

            0x90..=0x9F => self.sub(bus), // This includes SUB and SBB (subtrahend included in fn)

            0xA0..=0xA7 => self.ana(bus),
            0xA8..=0xAF => self.xra(bus),

            0xB0..=0xB7 => self.ora(bus),
            0xB8..=0xBF => self.cmp(bus),

            0xC0 => self.rnz(bus),                             // 11 or 5 cycles
            0xC1 => self.pop(Registers::BC, bus),
            0xC2 => self.jnz(dl, dh),
            0xC3 | 0xCB => self.jmp(dl, dh),
            0xC4 => self.cnz(dl, dh, bus),                       // 17 or 11 cycles
            0xC5 => self.push(self.c, self.b, bus),
            0xC6 | 0xCE =>  self.adi_aci(dl),
            0xC7 => self.rst(0, bus),
            0xC8 => self.rz(bus),                              // 11 or 5 cycles
            0xC9 | 0xD9 => self.ret(bus),
            0xCA => self.jz(dl, dh),
            0xCC => self.cz(dl, dh, bus),                        // 17 or 11 cycles
            0xCD | 0xDD | 0xED | 0xFD => self.call(dl, dh, bus),
            0xCF => self.rst(1, bus),

            0xD0 => self.rnc(bus),                             // 11 or 5 cycles
            0xD1 => self.pop(Registers::DE, bus),
            0xD2 => self.jnc(dl, dh),
            0xD3 => self.data_out(dl),
            0xD4 => self.cnc(dl, dh, bus),                       // 17 or 11 cycles
            0xD5 => self.push(self.e, self.d, bus),
            0xD7 => self.rst(2, bus),
            0xD8 => self.rc(bus),                              // 11 or 5 cycles
            0xDA => self.jc(dl, dh),
            0xDB => self.data_in(dl),               
            0xDC => self.cc(dl, dh, bus),                        // 17 or 11 cycles
            0xDF => self.rst(3, bus),

            0xE0 => self.rpo(bus),                             // 11 or 5 cycles
            0xE1 => self.pop(Registers::HL, bus),
            0xE2 => self.jpo(dl, dh),
            0xE3 => self.xthl(bus),
            0xE4 => self.cpo(dl, dh, bus),                       // 17 or 11 cycles
            0xE5 => self.push(self.l, self.h, bus),
            0xE6 => self.ani(dl),
            0xE7 => self.rst(4, bus),
            0xE8 => self.rpe(bus),                             // 11 or 5 cycles
            0xE9 => self.pchl(),
            0xEA => self.jpe(dl, dh),
            0xEB => self.xchg(),
            0xEC => self.cpe(dl, dh, bus),                       // 17 or 11 cycles
            0xEF => self.rst(5, bus),

            0xF0 => self.rp(bus),                              // 11 or 5 cycles
            0xF1 => self.pop(Registers::SW, bus),
            0xF2 => self.jp(dl, dh),
            0xF3 => self.di(),
            0xF4 => self.cp(dl, dh, bus),                        // 17 or 11 cycles
            0xF5 => self.push(self.get_flags(), self.a, bus),
            0xF7 => self.rst(6, bus),
            0xF8 => self.rm(bus),                              // 11 or 5 cycles
            0xF9 => self.sphl(),
            0xFA => self.jm(dl, dh),
            0xFB => self.ei(),
            0xFC => self.cm(dl, dh, bus),                        // 17 or 11 cycles
            0xFE => self.cpi(dl),
            0xFF => self.rst(7, bus),

            _ => Err(format!(
                "Unable to process UNKNOWN OPCODE: {}",
                self.current_instruction
            )),
        };

        match opcode_result {
            Ok(cycles_ran) => {
                // If PC has not changed due to a jump, etc, let's advance it like normal:
                if self.pc == pc_before {
                    self.pc += self.current_instruction.size;
                }
 
                Ok(cycles_ran)
            }
            Err(e) => Err(e),
        }
    }

    // Returns a usize location in memory designed by the H and L registers
    pub fn get_addr_pointer(&mut self) -> usize {
        usize::from(u16::from(self.h) << 8 | u16::from(self.l))
    }

    // Returns a tuple with dl and dh populated, if able to.  Uses the values
    // located in memory at PC+1 and PC+2
    fn get_data_pair(&mut self, bus: &Bus) -> (u8, u8) {
        let dl = bus.read(self.pc + 1);
        let dh = bus.read(self.pc + 2);
        (dl, dh)
    }

    // This function simply provides convenience when testing and we need to
    // execute an instruction along with its DL and DH values, which will be read
    // when the cpu gets to the whole "run opcode" ...thing.
    // This will overwrite what is in PC, etc.
    #[allow(unused)] // It's used in testing...
    pub fn prep_instr_and_data(&mut self, bus: &mut Bus, opcode: u8, dl: u8, dh: u8) {
        // TODO: Make this use memory as a module with ability to write by range, and freakout.
        self.current_instruction = Instruction::new(opcode);
        bus.write(self.pc + 1, dl);
        bus.write(self.pc + 2, dh);
    }

    // Returns a paired register such as HL or BC.
    // Pass to the function the beginning register for the pair
    // Returned value will be a u16 value
    #[must_use]
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
        }
    }

    /// Sets a flag to a provided boolean value
    pub fn update_flag(&mut self, mask: u8, value: bool) {
        if value {
            self.set_flag(mask);
        } else {
            self.reset_flag(mask);
        }
    }

    /// Sets a flag using a bitwise OR operation
    /// Mask of 2 (00100)
    /// if flags = 10010 new value will be 10110
    pub fn set_flag(&mut self, mask: u8) {
        self.flags |= mask;
    }

    /// Resets a flag using bitwise AND operation
    /// Mask of 2 (00100)
    /// if flags = 11111 new value will be 11011
    pub fn reset_flag(&mut self, mask: u8) {
        self.flags &= !mask;
    }

    /// Returns the current flag values, also known as the PSW
    #[must_use]
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    // Returns true if a flag is set
    pub fn test_flag(&mut self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    /// Returns the binary value of a flag, as a u8 for various ops.
    /// TODO: I don't like how this is not returning a single bit, instead of the u8.
    pub fn get_flag(&mut self, mask: u8) -> u8 {
        u8::from(self.test_flag(mask))
    }

    #[must_use]
    pub fn get_all_registers(&self) -> (&usize, &u16, &u8, &u8, &u8, &u8, &u8, &u8, &u8) {
        (&self.pc, &self.sp, &self.a, &self.b, &self.c, &self.d, &self.e, &self.h, &self.l)
    }

    // Computes and sets the mask of flags for a supplied value
    // sets flags: Zero, Sign, Parity, Carry, and Auxiliary Carry
    // If provided, it will also set Overflow and Aux_Carry, resetting them otherwise
    pub fn update_flags(&mut self, val: u8, overflow: Option<bool>, aux_carry: Option<bool>) {
        if val == 0 {
            self.set_flag(FLAG_ZERO);
        } else {
            self.reset_flag(FLAG_ZERO);
        }

        if get_sign(val) {
            self.set_flag(FLAG_SIGN); // A negative number
        } else {
            self.reset_flag(FLAG_SIGN); // A positive number
        }

        if get_parity(val.into()) {
            self.set_flag(FLAG_PARITY);
        } else {
            self.reset_flag(FLAG_PARITY);
        }

        if let Some(of) = overflow {
            if of {
                self.set_flag(FLAG_CARRY);
            } else {
                self.reset_flag(FLAG_CARRY);
            }
        }

        if let Some(ac) = aux_carry {
            if ac {
                self.set_flag(FLAG_AUXCARRY);
            } else {
                self.reset_flag(FLAG_AUXCARRY);
            }
        }
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }
}

/// Makes a memory pointer by simply concatenating the two values
/// For instance:
/// `make_pointer(0xFF,0xAA)` will return 0xAAFF
#[must_use]
#[allow(unused)]
pub fn make_pointer(dl: u8, dh: u8) -> u16 {
    (u16::from(dh) << 8 | u16::from(dl))
}

// If number of ones in a number's binary representation is even,
// parity flag is TRUE (1) else it is FALSE (0)
#[must_use]
#[allow(unused)]
pub fn get_parity(v: u16) -> bool {
    v.count_ones().is_multiple_of(2)
}

// Returns true if MSB = 1
#[must_use]
#[allow(unused)]
pub fn get_sign(x: u8) -> bool {
    (0b1000_0000 & x) != 0
}

// Returns true if an addition will case an aux carry
// value: the value we are trying to add to source
// source: the source that value is added to
#[must_use]
#[allow(unused)]
pub fn will_ac(value: u8, source: u8) -> bool {
    ((value & 0x0F) + (source & 0x0F)) & 0x10 == 0x10
}
