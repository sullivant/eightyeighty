// CPU Constants
pub const OPCODE_SIZE: usize = 1;
pub const RAM_SIZE: usize = 0xFFFF;

// Flags and their order/bitmasks
// S - Sign Flag
// Z - Zero Flag
// 0 - Not used, always zero
// A - also called AC, Auxiliary Carry Flag
// 0 - Not used, always zero
// P - Parity Flag
// 1 - Not used, always one
// C - Carry Flag
pub const FLAG_SIGN: u8 = 0b1000_0000;
pub const FLAG_ZERO: u8 = 0b0100_0000;
pub const FLAG_AUXCARRY: u8 = 0b0001_0000;
pub const FLAG_PARITY: u8 = 0b0000_0100;
pub const FLAG_CARRY: u8 = 0b0000_0001;

// Window and display concerns
pub const DISP_WIDTH: u16 = 640; // Overall width/height
pub const DISP_HEIGHT: u16 = 480;
pub const EMU_WIDTH: u16 = 224; // Emulator display area width/height
pub const EMU_HEIGHT: u16 = 256;
pub const CELL_SIZE: u16 = 2; // The size of a "cell" or pixel
pub const LINE_SPACE: u16 = 20; // Space between lines of text
