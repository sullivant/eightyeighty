use std::fmt;

mod instructions;
mod tests;

use crate::{
    constants::{FLAG_AUXCARRY, FLAG_CARRY, FLAG_PARITY, FLAG_SIGN, FLAG_ZERO, OPCODE_SIZE},
    memory::Memory,
};
use instructions::Instruction;

#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct CPU {
    // Memory
    pub memory: Memory,

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

    // If we are in single step mode, we wait until "ok_to_step" is true
    pub single_step_mode: bool,
    pub ok_to_step: bool,
    pub ok_to_print: bool,
    pub tick_happened: bool, // Did we actually process a tick last time?  Used when single stepping

    // A flag to indicate that we do not wish to execute, probably just printing disassembly
    pub nop: bool,

    pub interrupts: bool, // A flag to indicate we respond to interrupts (see: opcodes EI/DI)

    pub cycle_count: usize, // Cycle count
    pub current_instruction: Instruction,
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

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CYCLES:{:#08X} PC:{:#06X} SP:{:#06X} A:{:#06X} B:{:#04X} C:{:#04X} D:{:#04X} E:{:#04X} H:{:#04X} L:{:#04X} sp $[{:#06X}]={:#04X} sp+1 $[{:06X}]={:#04X} sp+2 $[{:06X}]={:#04X}",
            self.cycle_count, self.pc, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l,self.sp,self.memory.read(usize::from(self.sp)).unwrap(),self.sp+1,self.memory.read(usize::from(self.sp+1)).unwrap(),self.sp+2,self.memory.read(usize::from(self.sp+2)).unwrap()
        )
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    #[must_use]
    pub fn new() -> CPU {
        CPU {
            //memory: [0; RAM_SIZE],
            memory: Memory::new(),
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

            single_step_mode: false,
            ok_to_step: true,
            ok_to_print: true,
            tick_happened: false,

            nop: false,
            interrupts: false,
            cycle_count: 1,
            current_instruction: Instruction::new(0x00),
        }
    }

    // Reads an instruction at ProgramCounter
    pub fn read_instruction(&mut self) -> Instruction {
        let opcode = self.memory.read(self.pc).unwrap_or(0);

        Instruction::new(opcode) // new() will fill in the rest..
    }

    /// Gathers a word from memory based on program counter location,
    /// then passes it along to the ``run_opcode()`` function
    /// On successful tick, returns the program counter value that was run
    /// On unsuccessful tick, returns an error
    ///
    /// # Errors
    /// Will return an error if necessary
    /// # Panics
    /// Will panic if an error happens
    pub fn tick(&mut self) -> Result<(), String> {
        let opcode = self.read_instruction(); // Gather the current opcode to run, based on PC's location
        self.current_instruction = opcode;

        // If we are in a STOPPED state, no action is necessary
        // This will be "unstopped" when an interrupt occurs
        if self.nop {
            return Ok(());
        }

        // TODO: Make this respect "disassemble mode"

        // Print the opcode we are going to run with the current CPU state alongside.
        // TODO: Have this also gather potential DL,DH values
        if self.ok_to_print {
            println!("{} @ {}", self.current_instruction, self);
        }

        // While we are in single step mode, let's just return,
        // changing nothing about the PC, etc.
        if self.single_step_mode && !self.ok_to_step {
            self.ok_to_print = false;
            return Ok(());
        }

        // If we get this far, we need to reset "ok_to_step" to false for next run!
        if self.single_step_mode {
            self.ok_to_print = true;
            self.ok_to_step = false;
        }

        self.cycle_count += 1;

        // If we are not ok after running the opcode, we will error
        match self.run_opcode() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // Gathers the data necessary for the instruction and
    // calls out to the appropriate instruction operation to
    // perform the thing...
    #[allow(clippy::too_many_lines)]
    pub fn run_opcode(&mut self) -> Result<(), String> {
        let (dl, dh) = match self.get_data_pair() {
            Ok(value) => value,
            Err(value) => return value,
        };

        // Do the actual run of the opcode and return the result
        let opcode_result = match self.current_instruction.opcode {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Ok(()),

            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => self.mvi(dl),

            0x09 | 0x19 | 0x29 | 0x39 => {
                self.op_dad();
                Ok(())
            }

            0x01 => self.lxi(Registers::BC, dl, dh),
            0x02 => self.op_stax(Registers::BC), // STAX (BC)
            0x03 => {
                self.inx(Registers::BC);
                Ok(())
            }
            0x04 => self.op_inr(Registers::B),
            0x05 => self.op_dcr(Registers::B),
            0x0B => {
                self.dcx(Registers::BC);
                Ok(())
            }
            0x0C => self.op_inr(Registers::C),
            0x0D => self.op_dcr(Registers::C),

            0x0F => {
                self.op_rrc_rar(true);
                Ok(())
            } // RRC

            0x11 => self.lxi(Registers::DE, dl, dh),
            0x12 => self.op_stax(Registers::DE), // STAX (DE)
            0x13 => {
                self.inx(Registers::DE);
                Ok(())
            }
            0x14 => self.op_inr(Registers::D),
            0x15 => self.op_dcr(Registers::D),
            0x1B => {
                self.dcx(Registers::DE);
                Ok(())
            }
            0x1C => self.op_inr(Registers::E),
            0x1D => self.op_dcr(Registers::E),
            0x1F => {
                self.op_rrc_rar(false);
                Ok(())
            } // RAR

            0x21 => self.lxi(Registers::HL, dl, dh),
            0x2A => self.lhld(dl, dh),
            0x23 => {
                self.inx(Registers::HL);
                Ok(())
            }
            0x24 => self.op_inr(Registers::H),
            0x25 => self.op_dcr(Registers::H),
            0x27 => {
                self.op_daa();
                Ok(())
            }
            0x2B => {
                self.dcx(Registers::HL);
                Ok(())
            }
            0x2C => self.op_inr(Registers::L),
            0x2D => self.op_dcr(Registers::L),

            0x31 => self.lxi(Registers::SP, dl, dh),
            0x32 => self.op_sta(dl, dh), // STA (adr)<-A
            0x33 => {
                self.inx(Registers::SP);
                Ok(())
            }
            0x34 => self.op_inr(Registers::HL),
            0x35 => self.op_dcr(Registers::HL),
            0x3B => {
                self.dcx(Registers::SP);
                Ok(())
            }
            0x3C => self.op_inr(Registers::A),
            0x3D => self.op_dcr(Registers::A),

            0x40 => self.mov(Registers::B, Registers::B), // MOV B <- B
            0x41 => self.mov(Registers::B, Registers::C), // MOV B <- C
            0x42 => self.mov(Registers::B, Registers::D), // MOV B <- D
            0x43 => self.mov(Registers::B, Registers::E), // MOV B <- E
            0x44 => self.mov(Registers::B, Registers::H), // MOV B <- H
            0x45 => self.mov(Registers::B, Registers::L), // MOV B <- L
            0x46 => self.mov(Registers::B, Registers::HL), // MOV B <- (HL)
            0x47 => self.mov(Registers::B, Registers::A), // MOV B <- A
            0x48 => self.mov(Registers::C, Registers::B), // MOV C <- B
            0x49 => self.mov(Registers::C, Registers::C), // MOV C <- C
            0x4A => self.mov(Registers::C, Registers::D), // MOV C <- D
            0x4B => self.mov(Registers::C, Registers::E), // MOV C <- E
            0x4C => self.mov(Registers::C, Registers::H), // MOV C <- H
            0x4D => self.mov(Registers::C, Registers::L), // MOV C <- L
            0x4E => self.mov(Registers::C, Registers::HL), // MOV C <- HL
            0x4F => self.mov(Registers::C, Registers::A), // MOV C <- A

            0x50 => self.mov(Registers::D, Registers::B), // MOV D <- B
            0x51 => self.mov(Registers::D, Registers::C), // MOV D <- C
            0x52 => self.mov(Registers::D, Registers::D), // MOV D <- D
            0x53 => self.mov(Registers::D, Registers::E), // MOV D <- E
            0x54 => self.mov(Registers::D, Registers::H), // MOV D <- H
            0x55 => self.mov(Registers::D, Registers::L), // MOV D <- L
            0x56 => self.mov(Registers::D, Registers::HL), // MOV D <- (HL)
            0x57 => self.mov(Registers::D, Registers::A), // MOV D <- A
            0x58 => self.mov(Registers::E, Registers::B), // MOV E <- B
            0x59 => self.mov(Registers::E, Registers::C), // MOV E <- C
            0x5A => self.mov(Registers::E, Registers::D), // MOV E <- D
            0x5B => self.mov(Registers::E, Registers::E), // MOV E <- E
            0x5C => self.mov(Registers::E, Registers::H), // MOV E <- H
            0x5D => self.mov(Registers::E, Registers::L), // MOV E <- L
            0x5E => self.mov(Registers::E, Registers::HL), // MOV E <- HL
            0x5F => self.mov(Registers::E, Registers::A), // MOV E <- A

            0x60 => self.mov(Registers::H, Registers::B), // MOV H <- B
            0x61 => self.mov(Registers::H, Registers::C), // MOV H <- C
            0x62 => self.mov(Registers::H, Registers::D), // MOV H <- D
            0x63 => self.mov(Registers::H, Registers::E), // MOV H <- E
            0x64 => self.mov(Registers::H, Registers::H), // MOV H <- H
            0x65 => self.mov(Registers::H, Registers::L), // MOV H <- L
            0x66 => self.mov(Registers::H, Registers::HL), // MOV H <- (HL)
            0x67 => self.mov(Registers::H, Registers::A), // MOV H <- A
            0x68 => self.mov(Registers::L, Registers::B), // MOV L <- B
            0x69 => self.mov(Registers::L, Registers::C), // MOV L <- C
            0x6A => self.mov(Registers::L, Registers::D), // MOV L <- D
            0x6B => self.mov(Registers::L, Registers::E), // MOV L <- E
            0x6C => self.mov(Registers::L, Registers::H), // MOV L <- H
            0x6D => self.mov(Registers::L, Registers::L), // MOV L <- L
            0x6E => self.mov(Registers::L, Registers::HL), // MOV L <- HL
            0x6F => self.mov(Registers::L, Registers::A), // MOV L <- A

            0x70 => self.mov(Registers::HL, Registers::B), // MOV M,B	1		(HL) <- B
            0x71 => self.mov(Registers::HL, Registers::C), // MOV M,C	1		(HL) <- C
            0x72 => self.mov(Registers::HL, Registers::D), // MOV M,D	1		(HL) <- D
            0x73 => self.mov(Registers::HL, Registers::E), // MOV M,E	1		(HL) <- E
            0x74 => self.mov(Registers::HL, Registers::H), // MOV M,H	1		(HL) <- H
            0x75 => self.mov(Registers::HL, Registers::L), // MOV M,L	1		(HL) <- L
            0x76 => self.hlt(),
            0x77 => self.mov(Registers::HL, Registers::A), // MOV M,A
            0x78 => self.mov(Registers::A, Registers::B),  // MOV A,B
            0x79 => self.mov(Registers::A, Registers::C),  // MOV A,C
            0x7A => self.mov(Registers::A, Registers::D),  // MOV A,D
            0x7B => self.mov(Registers::A, Registers::E),  // MOV A,E
            0x7C => self.mov(Registers::A, Registers::H),  // MOV A,H
            0x7D => self.mov(Registers::A, Registers::L),  // MOV A,L
            0x7E => self.mov(Registers::A, Registers::HL), // MOV A,(HL)
            0x7F => self.mov(Registers::A, Registers::A),  // MOV A,A

            0x80..=0x87 => self.op_add(),
            0x88..=0x8F => self.op_adc(),

            0x90..=0x9F => self.op_sub(), // This includes SUB and SBB (subtrahend included in fn)

            0xA0..=0xA7 => self.op_ana(),
            0xA8..=0xAF => self.op_xra(),

            0xB0..=0xB7 => self.op_ora(),
            0xB8..=0xBF => self.op_cmp(),

            0xC3 | 0xCB => self.jmp(dl, dh),

            0xC6 | 0xCE => {
                self.op_adi_aci(dl);
                Ok(())
            }

            0xD3 => self.data_out(dl),

            0xE6 => {
                self.op_ani(dl);
                Ok(())
            }

            0xFE => {
                self.op_cpi(dl);
                Ok(())
            }

            _ => Err(format!(
                "Unable to process UNKNOWN OPCODE: {}",
                self.current_instruction
            )),
        };

        match opcode_result {
            Ok(()) => {
                self.pc += self.current_instruction.size * OPCODE_SIZE;
                Ok(())
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
    fn get_data_pair(&mut self) -> Result<(u8, u8), Result<(), String>> {
        let dl = match self.memory.read(self.pc + 1) {
            Ok(v) => v,
            Err(e) => return Err(Err(e)),
        };
        let dh = match self.memory.read(self.pc + 2) {
            Ok(v) => v,
            Err(e) => return Err(Err(e)),
        };
        Ok((dl, dh))
    }

    pub fn toggle_single_step_mode(&mut self) {
        self.single_step_mode = !self.single_step_mode;

        self.ok_to_print = true;
    }

    pub fn disassemble(&mut self, val: bool) -> bool {
        self.disassemble = val;
        self.disassemble
    }

    pub fn nop(&mut self, val: bool) {
        self.nop = val;
    }

    // This function simply provides convenience when testing and we need to
    // execute an instruction along with its DL and DH values, which will be read
    // when the cpu gets to the whole "run opcode" ...thing.
    // This will overwrite what is in PC, etc.
    #[allow(unused)] // It's used in testing...
    pub fn prep_instr_and_data(&mut self, opcode: u8, dl: u8, dh: u8) {
        // TODO: Make this use memory as a module with ability to write by range, and freakout.
        self.current_instruction = Instruction::new(opcode);
        self.memory.write(self.pc + 1, dl);
        self.memory.write(self.pc + 2, dh);
    }

    // This allows for access to memory, by reference, from outside of the CPU
    pub fn memory(&mut self) -> &mut Memory {
        &mut self.memory
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
        };
    }

    // Sets a flag using a bitwise OR operation
    // Mask of 2 (00100)
    // if flags = 10010 new value will be 10110
    pub fn set_flag(&mut self, mask: u8) {
        self.flags |= mask;
    }

    // Resets a flag using bitwise AND operation
    // Mask of 2 (00100)
    // if flags = 11111 new value will be 11011
    pub fn reset_flag(&mut self, mask: u8) {
        self.flags &= !mask;
    }

    // Returns the current flag values
    #[must_use]
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    // Returns true if a flag is set
    pub fn test_flag(&mut self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    // Returns the binary value of a flag, as a u8 for various ops.
    pub fn get_flag(&mut self, mask: u8) -> u8 {
        u8::from(self.test_flag(mask))
    }

    #[must_use]
    pub fn get_registers(&self) -> (&usize, &u16, &u8, &u8, &u8) {
        (&self.pc, &self.sp, &self.h, &self.l, &self.b)
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
        };

        if let Some(ac) = aux_carry {
            if ac {
                self.set_flag(FLAG_AUXCARRY);
            } else {
                self.reset_flag(FLAG_AUXCARRY);
            }
        };
    }
}

// Makes a memory pointer by simply concatenating the two values
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
    v.count_ones() % 2 == 0
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
