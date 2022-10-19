// CPU Constants
pub const OPCODE_SIZE: usize = 1;
pub const RAM_SIZE: usize = 0xFFFF;

pub const HEADER: &str =
    "CYCLE  PC       Ins  S  l,   h,   sp      SZ0A0P1C  data(l,h)  B    Halt? : Command";

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

// OPCODE Descriptions
pub const OP_NOP: &str = "NOP";
pub const OP_RLC: &str = "RLC";
pub const OP_RRC: &str = "RRC";
pub const OP_SHLD: &str = "SHLD";
pub const OP_STA: &str = "STA";
pub const OP_RAL: &str = "RAL";
pub const OP_RAR: &str = "RAR";
pub const OP_DAA: &str = "DAA";
pub const OP_LHLD: &str = "LHLD";
pub const OP_CMA: &str = "CMA";
pub const OP_STC: &str = "STC";
pub const OP_LDA: &str = "LDA";
pub const OP_CMC: &str = "CMC";
pub const OP_HLT: &str = "HLT";
pub const OP_RNZ: &str = "RNZ";
pub const OP_JNZ: &str = "JNZ";
pub const OP_CNZ: &str = "CNZ";
pub const OP_ADI: &str = "ADI";
pub const OP_RC: &str = "RC";
pub const OP_RET: &str = "RET";
pub const OP_JZ: &str = "JZ";
pub const OP_CZ: &str = "CZ";
pub const OP_CALL: &str = "CALL";
pub const OP_ACI: &str = "ACI";
pub const OP_RNC: &str = "RNC";
pub const OP_JNC: &str = "JNC";
pub const OP_JC: &str = "JC";
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
pub const OP_RZ: &str = "RZ";
pub const OP_SUI: &str = "SUI";
pub const OP_IN: &str = "IN";
pub const OP_SBI: &str = "SBI";
pub const OP_JPO: &str = "JPO";
pub const OP_XTHL: &str = "XTHL";
pub const OP_PCHL: &str = "PCHL";
pub const OP_ANI: &str = "ANI";
pub const OP_JPE: &str = "JPE";
pub const OP_XRI: &str = "XRI";
pub const OP_JMP_16: &str = "JMP a16";
pub const OP_JM_16: &str = "JM IF a16";
pub const OP_JP_16: &str = "JM a16";
pub const OP_DI: &str = "DI";
pub const OP_ORI: &str = "ORI";
pub const OP_SPHL: &str = "SPHL";
pub const OP_EI: &str = "EI";
pub const OP_CM: &str = "CM";

pub const OP_STAX_B: &str = "STAX B";
pub const OP_LDAX_B: &str = "LDAX B";

pub const OP_STAX_D: &str = "STAX D";
pub const OP_LDAX_D: &str = "LDAX D";

pub const OP_LXI_B: &str = "LXI B";
pub const OP_LXI_D: &str = "LXI D";
pub const OP_LXI_H: &str = "LXI H";
pub const OP_LXI_SP: &str = "LXI SP";

pub const OP_INX_B: &str = "INX B";
pub const OP_INX_D: &str = "INX D";
pub const OP_INX_H: &str = "INX H";
pub const OP_INX_SP: &str = "INX SP";

pub const OP_DAD_B: &str = "DAD B";
pub const OP_DAD_D: &str = "DAD D";
pub const OP_DAD_H: &str = "DAD H";
pub const OP_DAD_SP: &str = "DAD SP";

pub const OP_DCX_B: &str = "DCX B";
pub const OP_DCX_D: &str = "DCX D";
pub const OP_DCX_H: &str = "DCX H";
pub const OP_DCX_SP: &str = "DCX SP";

pub const OP_INR_B: &str = "INR B";
pub const OP_INR_C: &str = "INR C";
pub const OP_INR_D: &str = "INR D";
pub const OP_INR_E: &str = "INR E";
pub const OP_INR_H: &str = "INR H";
pub const OP_INR_L: &str = "INR L";
pub const OP_INR_M: &str = "INR M";
pub const OP_INR_A: &str = "INR A";

pub const OP_DCR_B: &str = "DCR B";
pub const OP_DCR_C: &str = "DCR C";
pub const OP_DCR_D: &str = "DCR D";
pub const OP_DCR_E: &str = "DCR E";
pub const OP_DCR_H: &str = "DCR H";
pub const OP_DCR_L: &str = "DCR L";
pub const OP_DCR_M: &str = "DCR M";
pub const OP_DCR_A: &str = "DCR A";

pub const OP_MVI_B: &str = "MVI B";
pub const OP_MVI_C: &str = "MVI C";
pub const OP_MVI_D: &str = "MVI D";
pub const OP_MVI_E: &str = "MVI E";
pub const OP_MVI_H: &str = "MVI H";
pub const OP_MVI_L: &str = "MVI L";
pub const OP_MVI_M: &str = "MVI M";
pub const OP_MVI_A: &str = "MVI A";

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

pub const OP_ADD_B: &str = "ADD B";
pub const OP_ADD_C: &str = "ADD C";
pub const OP_ADD_D: &str = "ADD D";
pub const OP_ADD_E: &str = "ADD E";
pub const OP_ADD_H: &str = "ADD H";
pub const OP_ADD_L: &str = "ADD L";
pub const OP_ADD_M: &str = "ADD M";
pub const OP_ADD_A: &str = "ADD A";

pub const OP_ADC_B: &str = "ADC B";
pub const OP_ADC_C: &str = "ADC B";
pub const OP_ADC_D: &str = "ADC B";
pub const OP_ADC_E: &str = "ADC B";
pub const OP_ADC_H: &str = "ADC B";
pub const OP_ADC_L: &str = "ADC B";
pub const OP_ADC_M: &str = "ADC B";
pub const OP_ADC_A: &str = "ADC B";

pub const OP_SUB_B: &str = "SUB B";
pub const OP_SUB_C: &str = "SUB C";
pub const OP_SUB_D: &str = "SUB D";
pub const OP_SUB_E: &str = "SUB E";
pub const OP_SUB_H: &str = "SUB H";
pub const OP_SUB_L: &str = "SUB L";
pub const OP_SUB_M: &str = "SUB M";
pub const OP_SUB_A: &str = "SUB A";

pub const OP_SBB_B: &str = "SBB B";
pub const OP_SBB_C: &str = "SBB C";
pub const OP_SBB_D: &str = "SBB D";
pub const OP_SBB_E: &str = "SBB E";
pub const OP_SBB_H: &str = "SBB H";
pub const OP_SBB_L: &str = "SBB L";
pub const OP_SBB_M: &str = "SBB M";
pub const OP_SBB_A: &str = "SBB A";

pub const OP_ANA_B: &str = "ANA B";
pub const OP_ANA_C: &str = "ANA C";
pub const OP_ANA_D: &str = "ANA D";
pub const OP_ANA_E: &str = "ANA E";
pub const OP_ANA_H: &str = "ANA H";
pub const OP_ANA_L: &str = "ANA L";
pub const OP_ANA_M: &str = "ANA M";
pub const OP_ANA_A: &str = "ANA A";

pub const OP_XRA_B: &str = "XRA B";
pub const OP_XRA_C: &str = "XRA C";
pub const OP_XRA_D: &str = "XRA D";
pub const OP_XRA_E: &str = "XRA E";
pub const OP_XRA_H: &str = "XRA H";
pub const OP_XRA_L: &str = "XRA L";
pub const OP_XRA_M: &str = "XRA M";
pub const OP_XRA_A: &str = "XRA A";

pub const OP_ORA_B: &str = "ORA B";
pub const OP_ORA_C: &str = "ORA C";
pub const OP_ORA_D: &str = "ORA D";
pub const OP_ORA_E: &str = "ORA E";
pub const OP_ORA_H: &str = "ORA H";
pub const OP_ORA_L: &str = "ORA L";
pub const OP_ORA_M: &str = "ORA M";
pub const OP_ORA_A: &str = "ORA A";

pub const OP_CMP_B: &str = "CMP B";
pub const OP_CMP_C: &str = "CMP C";
pub const OP_CMP_D: &str = "CMP D";
pub const OP_CMP_E: &str = "CMP E";
pub const OP_CMP_H: &str = "CMP H";
pub const OP_CMP_L: &str = "CMP L";
pub const OP_CMP_M: &str = "CMP M";
pub const OP_CMP_A: &str = "CMP A";

pub const OP_RST_0: &str = "RST 0";
pub const OP_RST_1: &str = "RST 1";
pub const OP_RST_2: &str = "RST 2";
pub const OP_RST_3: &str = "RST 3";
pub const OP_RST_4: &str = "RST 4";
pub const OP_RST_5: &str = "RST 5";
pub const OP_RST_6: &str = "RST 6";
pub const OP_RST_7: &str = "RST 7";

pub const OP_PUSH_B: &str = "PUSH B";
pub const OP_PUSH_D: &str = "PUSH D";
pub const OP_PUSH_H: &str = "PUSH H";
pub const OP_PUSH_PSW: &str = "PUSH PSW";

pub const OP_POP_B: &str = "POP B";
pub const OP_POP_D: &str = "POP D";
pub const OP_POP_H: &str = "POP H";
pub const OP_POP_PSW: &str = "POP PSW";
