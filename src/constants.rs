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

// OPCODE Descriptions
pub const OP_NOP: &str = "NOP";
pub const OP_UNK: &str = "UNK";
pub const OP_RAL: &str = "RAL";
pub const OP_DAD: &str = "DAD";
pub const OP_LDAX: &str = "LDAX";
pub const OP_DCX: &str = "DCX";
pub const OP_INR: &str = "INR";
pub const OP_DCR: &str = "DCR";
pub const OP_MVI: &str = "MVI";
pub const OP_RAR: &str = "RAR";
pub const OP_LXI: &str = "LXI";
pub const OP_INX: &str = "INX";
pub const OP_DAA: &str = "DAA";
pub const OP_LHLD: &str = "LHLD";
pub const OP_CMA: &str = "CMA";
pub const OP_STA: &str = "STA";
pub const OP_STC: &str = "STC";
pub const OP_LDA: &str = "LDA";
pub const OP_CMC: &str = "CMC";
pub const OP_MOV: &str = "MOV";
pub const OP_MOV_BB: &str = "MOV B,B";
pub const OP_MOV_BC: &str = "MOV B,C";
pub const OP_MOV_BD: &str = "MOV B,D";
pub const OP_MOV_BE: &str = "MOV B,E";
pub const OP_MOV_BH: &str = "MOV B,H";
pub const OP_MOV_BL: &str = "MOV B,L";
pub const OP_MOV_BM: &str = "MOV B,M";
pub const OP_MOV_BA: &str = "MOV B,A";
pub const OP_MOV_CB: &str = "MOV C,B";
pub const OP_MOV_CC: &str = "MOV C,C";
pub const OP_MOV_CD: &str = "MOV C,D";
pub const OP_MOV_CE: &str = "MOV C,E";
pub const OP_MOV_CH: &str = "MOV C,H";
pub const OP_MOV_CL: &str = "MOV C,L";
pub const OP_MOV_CM: &str = "MOV C,M";
pub const OP_MOV_CA: &str = "MOV C,A";

pub const OP_MOV_DB: &str = "MOV D,B";
pub const OP_MOV_DC: &str = "MOV D,C";
pub const OP_MOV_DD: &str = "MOV D,D";
pub const OP_MOV_DE: &str = "MOV D,E";
pub const OP_MOV_DH: &str = "MOV D,H";
pub const OP_MOV_DL: &str = "MOV D,L";
pub const OP_MOV_DM: &str = "MOV D,M";
pub const OP_MOV_DA: &str = "MOV D,A";
pub const OP_MOV_EB: &str = "MOV E,B";
pub const OP_MOV_EC: &str = "MOV E,C";
pub const OP_MOV_ED: &str = "MOV E,D";
pub const OP_MOV_EE: &str = "MOV E,E";
pub const OP_MOV_EH: &str = "MOV E,H";
pub const OP_MOV_EL: &str = "MOV E,L";
pub const OP_MOV_EM: &str = "MOV E,M";
pub const OP_MOV_EA: &str = "MOV E,A";

pub const OP_MOV_HB: &str = "MOV H,B";
pub const OP_MOV_HC: &str = "MOV H,C";
pub const OP_MOV_HD: &str = "MOV H,D";
pub const OP_MOV_HE: &str = "MOV H,E";
pub const OP_MOV_HH: &str = "MOV H,H";
pub const OP_MOV_HL: &str = "MOV H,L";
pub const OP_MOV_HM: &str = "MOV H,M";
pub const OP_MOV_HA: &str = "MOV H,A";
pub const OP_MOV_LB: &str = "MOV L,B";
pub const OP_MOV_LC: &str = "MOV L,C";
pub const OP_MOV_LD: &str = "MOV L,D";
pub const OP_MOV_LE: &str = "MOV L,E";
pub const OP_MOV_LH: &str = "MOV L,H";
pub const OP_MOV_LL: &str = "MOV L,L";
pub const OP_MOV_LM: &str = "MOV L,M";
pub const OP_MOV_LA: &str = "MOV L,A";

pub const OP_MOV_MB: &str = "MOV M,B";
pub const OP_MOV_MC: &str = "MOV M,C";
pub const OP_MOV_MD: &str = "MOV M,D";
pub const OP_MOV_ME: &str = "MOV M,E";
pub const OP_MOV_MH: &str = "MOV M,H";
pub const OP_MOV_ML: &str = "MOV M,L";
pub const OP_MOV_MA: &str = "MOV M,A";
pub const OP_MOV_AB: &str = "MOV A,B";
pub const OP_MOV_AC: &str = "MOV A,C";
pub const OP_MOV_AD: &str = "MOV A,D";
pub const OP_MOV_AE: &str = "MOV A,E";
pub const OP_MOV_AH: &str = "MOV A,H";
pub const OP_MOV_AL: &str = "MOV A,L";
pub const OP_MOV_AM: &str = "MOV A,M";
pub const OP_MOV_AA: &str = "MOV A,A";

pub const OP_HLT: &str = "HLT";
pub const OP_ADD: &str = "ADD";
pub const OP_ADC: &str = "ADC";
pub const OP_SUB: &str = "SUB";
pub const OP_SBB: &str = "SBB";
pub const OP_ANA: &str = "ANA";
pub const OP_XRA: &str = "XRA";
pub const OP_ORA: &str = "ORA";
pub const OP_CMP: &str = "CMP";
pub const OP_RNZ: &str = "RNZ";
pub const OP_POP: &str = "POP";
pub const OP_JNZ: &str = "JNZ";
pub const OP_JMP: &str = "JMP";
pub const OP_CNZ: &str = "CNZ";
pub const OP_PUSH: &str = "PUSH";
pub const OP_ADI: &str = "ADI";
pub const OP_RST: &str = "RST";
pub const OP_RC: &str = "RC";
pub const OP_RET: &str = "RET";
pub const OP_JZ: &str = "JZ";
pub const OP_CZ: &str = "CZ";
pub const OP_CALL: &str = "CALL";
pub const OP_ACI: &str = "ACI";
pub const OP_RNC: &str = "RNC";
pub const OP_JNC: &str = "JNC";
pub const OP_OUT: &str = "OUT";
pub const OP_CNC: &str = "CNC";
pub const OP_CC: &str = "CC";
pub const OP_RPO: &str = "RPO";
pub const OP_CPO: &str = "CPO";
pub const OP_RPE: &str = "RPE";
pub const OP_XCHG: &str = "XCHG";
pub const OP_CPE: &str = "CPE";
pub const OP_RP: &str = "RP";
pub const OP_CP: &str = "CP";
pub const OP_RM: &str = "RM";
pub const OP_CPI: &str = "CPI";
