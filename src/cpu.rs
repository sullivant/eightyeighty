use std::{fmt, fs::File, io::Read};

mod instructions;
mod tests;

use crate::constants::{OPCODE_SIZE, RAM_SIZE};
use instructions::Instruction;

#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct CPU {
    // Memory
    pub memory: [u8; RAM_SIZE], // TODO: Make memory its own mod, able to get/set by range

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
            self.cycle_count, self.pc, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l,self.sp,self.memory[usize::from(self.sp)],self.sp+1,self.memory[usize::from(self.sp+1)],self.sp+2,self.memory[usize::from(self.sp+2)]
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

    /// Load the ROM file into memory, starting at ``start_index``
    /// Returns a tuple containing the index we started at and where we
    /// actually finished at.
    ///
    /// # Errors
    /// Will return a standard io Error if necessary
    /// # Panics
    /// If the error happens, this will cause the function to panic
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

    // Reads an instruction at ProgramCounter
    pub fn read_instruction(&mut self) -> Instruction {
        let opcode = match self.memory.get(self.pc) {
            Some(&v) => v,
            None => 0,
        };

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
    pub fn run_opcode(&mut self) -> Result<(), String> {
        // Gather our data bytes - up to two, if necessary
        // We may not use both of these, though
        let dl = match self.memory.get(self.pc + 1) {
            Some(&v) => v,
            None => 0,
        };
        let dh = match self.memory.get(self.pc + 2) {
            Some(&v) => v,
            None => 0,
        };

        // Do the actual run of the opcode and return the result
        let opcode_result = match self.current_instruction.opcode {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Ok(()),

            0x01 => self.lxi(Registers::B, dl, dh),

            0x2A => self.lhld(dl, dh),

            0x76 => self.hlt(),

            0xD3 => self.data_out(dl),

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
    pub fn prep_instr_and_data(&mut self, opcode: u8, dl: u8, dh: u8) {
        // TODO: Make this use memory as a module with ability to write by range, and freakout.
        self.current_instruction = Instruction::new(opcode);
        self.memory[self.pc+1] = dl;
        self.memory[self.pc+2] = dh;
    }

}

// Makes a memory pointer by simply concatenating the two values
#[must_use]
#[allow(unused)]
pub fn make_pointer(dl: u8, dh: u8) -> usize {
    usize::from(u16::from(dh) << 8 | u16::from(dl))
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

