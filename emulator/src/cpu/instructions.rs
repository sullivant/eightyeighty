use std::fmt;

mod artithmetic;
mod jump_call;
mod load_store_move;
mod misc;
use serde::{Deserialize, Serialize};

#[allow(clippy::wildcard_imports)]
use crate::constants::*;

#[allow(unused)]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: u8,         // The Hex value of the instruction
    pub size: usize,        // The size of the instruction, may include DH, DL
    pub cycles: u8,         // Number of CPU cycles this instr took
    pub text: &'static str, // Found in constansts
}

// impl Default for Instruction {
//     fn default() -> Self {
//         Self::new(0x00)
//     }
// }

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#04X} {:02}/{:02} {:10}",
            self.opcode, self.size, self.cycles, self.text
        )
    }
}

impl Instruction {
    #[allow(clippy::too_many_lines)]
    pub const fn new(opcode: u8) -> Instruction {
        // Gather size, cycles, and text
        let info = match opcode {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => (1, 4, OP_NOP),
            0x01 => (3, 10, OP_LXI_B),
            0x02 => (1, 7, OP_STAX_B),
            0x03 => (1, 5, OP_INX_B),
            0x04 => (1, 5, OP_INR_B),
            0x05 => (1, 5, OP_DCR_B),
            0x06 => (2, 7, OP_MVI_B),
            0x07 => (1, 4, OP_RLC),
            0x09 => (1, 10, OP_DAD_B),
            0x0A => (1, 7, OP_LDAX_B),
            0x0B => (1, 5, OP_DCX_B),
            0x0C => (1, 5, OP_INR_C),
            0x0D => (1, 5, OP_DCR_C),
            0x0E => (2, 7, OP_MVI_C),
            0x0F => (1, 4, OP_RRC),

            0x11 => (3, 10, OP_LXI_D),
            0x12 => (1, 7, OP_STAX_D),
            0x13 => (1, 5, OP_INX_D),
            0x14 => (1, 5, OP_INR_D),
            0x15 => (1, 5, OP_DCR_D),
            0x16 => (2, 7, OP_MVI_D),
            0x17 => (1, 4, OP_RAL),
            0x19 => (1, 10, OP_DAD_D),
            0x1A => (1, 7, OP_LDAX_D),
            0x1B => (1, 5, OP_DCX_D),
            0x1C => (1, 5, OP_INR_E),
            0x1D => (1, 5, OP_DCR_E),
            0x1E => (2, 7, OP_MVI_E),
            0x1F => (1, 4, OP_RAR),

            0x21 => (2, 10, OP_LXI_H),
            0x22 => (3, 16, OP_SHLD),
            0x23 => (1, 5, OP_INX_H),
            0x24 => (1, 5, OP_INR_H),
            0x25 => (1, 5, OP_DCR_H),
            0x26 => (2, 7, OP_MVI_H),
            0x27 => (1, 4, OP_DAA),
            0x29 => (1, 10, OP_DAD_H),
            0x2A => (3, 16, OP_LHLD),
            0x2B => (1, 5, OP_DCX_H),
            0x2C => (1, 5, OP_INR_L),
            0x2D => (1, 5, OP_DCR_L),
            0x2E => (2, 7, OP_MVI_L),
            0x2F => (1, 4, OP_CMA),

            0x31 => (3, 10, OP_LXI_SP),
            0x32 => (3, 13, OP_STA),
            0x33 => (1, 5, OP_INX_SP),
            0x34 => (1, 10, OP_INR_M),
            0x35 => (1, 10, OP_DCR_M),
            0x36 => (2, 10, OP_MVI_M),
            0x37 => (1, 4, OP_STC),
            0x39 => (1, 10, OP_DAD_SP),
            0x3A => (3, 13, OP_LDA),
            0x3B => (1, 5, OP_DCX_SP),
            0x3C => (1, 5, OP_INR_A),
            0x3D => (1, 5, OP_DCR_A),
            0x3E => (2, 7, OP_MVI_A),
            0x3F => (1, 4, OP_CMC),
            0x40 => (1, 5, OP_MOV_BB),
            0x41 => (1, 5, OP_MOV_BC),
            0x42 => (1, 5, OP_MOV_BD),
            0x43 => (1, 5, OP_MOV_BE),
            0x44 => (1, 5, OP_MOV_BH),
            0x45 => (1, 5, OP_MOV_BL),
            0x46 => (1, 7, OP_MOV_BM),
            0x47 => (1, 5, OP_MOV_BA),
            0x48 => (1, 5, OP_MOV_CB),
            0x49 => (1, 5, OP_MOV_CC),
            0x4A => (1, 5, OP_MOV_CD),
            0x4B => (1, 5, OP_MOV_CE),
            0x4C => (1, 5, OP_MOV_CH),
            0x4D => (1, 5, OP_MOV_CL),
            0x4E => (1, 7, OP_MOV_CM),
            0x4F => (1, 5, OP_MOV_CA),

            0x50 => (1, 5, OP_MOV_DB),
            0x51 => (1, 5, OP_MOV_DC),
            0x52 => (1, 5, OP_MOV_DD),
            0x53 => (1, 5, OP_MOV_DE),
            0x54 => (1, 5, OP_MOV_DH),
            0x55 => (1, 5, OP_MOV_DL),
            0x56 => (1, 7, OP_MOV_DM),
            0x57 => (1, 5, OP_MOV_DA),
            0x58 => (1, 5, OP_MOV_EB),
            0x59 => (1, 5, OP_MOV_EC),
            0x5A => (1, 5, OP_MOV_ED),
            0x5B => (1, 5, OP_MOV_EE),
            0x5C => (1, 5, OP_MOV_EH),
            0x5D => (1, 5, OP_MOV_EL),
            0x5E => (1, 7, OP_MOV_EM),
            0x5F => (1, 5, OP_MOV_EA),

            0x60 => (1, 5, OP_MOV_HB),
            0x61 => (1, 5, OP_MOV_HC),
            0x62 => (1, 5, OP_MOV_HD),
            0x63 => (1, 5, OP_MOV_HE),
            0x64 => (1, 5, OP_MOV_HH),
            0x65 => (1, 5, OP_MOV_HL),
            0x66 => (1, 7, OP_MOV_HM),
            0x67 => (1, 5, OP_MOV_HA),
            0x68 => (1, 5, OP_MOV_LB),
            0x69 => (1, 5, OP_MOV_LC),
            0x6A => (1, 5, OP_MOV_LD),
            0x6B => (1, 5, OP_MOV_LE),
            0x6C => (1, 5, OP_MOV_LH),
            0x6D => (1, 5, OP_MOV_LL),
            0x6E => (1, 7, OP_MOV_LM),
            0x6F => (1, 5, OP_MOV_LA),

            0x70 => (1, 7, OP_MOV_MB),
            0x71 => (1, 7, OP_MOV_MC),
            0x72 => (1, 7, OP_MOV_MD),
            0x73 => (1, 7, OP_MOV_ME),
            0x74 => (1, 7, OP_MOV_MH),
            0x75 => (1, 7, OP_MOV_ML),
            0x76 => (1, 7, OP_HLT),
            0x77 => (1, 7, OP_MOV_MA),
            0x78 => (1, 5, OP_MOV_AB),
            0x79 => (1, 5, OP_MOV_AC),
            0x7A => (1, 5, OP_MOV_AD),
            0x7B => (1, 5, OP_MOV_AE),
            0x7C => (1, 5, OP_MOV_AH),
            0x7D => (1, 5, OP_MOV_AL),
            0x7E => (1, 7, OP_MOV_AM),
            0x7F => (1, 5, OP_MOV_AA),

            0x80 => (1, 4, OP_ADD_B),
            0x81 => (1, 4, OP_ADD_C),
            0x82 => (1, 4, OP_ADD_D),
            0x83 => (1, 4, OP_ADD_E),
            0x84 => (1, 4, OP_ADD_H),
            0x85 => (1, 4, OP_ADD_L),
            0x86 => (1, 7, OP_ADD_M),
            0x87 => (1, 4, OP_ADD_A),
            0x88 => (1, 4, OP_ADC_B),
            0x89 => (1, 4, OP_ADC_C),
            0x8A => (1, 4, OP_ADC_D),
            0x8B => (1, 4, OP_ADC_E),
            0x8C => (1, 4, OP_ADC_H),
            0x8D => (1, 4, OP_ADC_L),
            0x8E => (1, 7, OP_ADC_M),
            0x8F => (1, 4, OP_ADC_A),

            0x90 => (1, 4, OP_SUB_B),
            0x91 => (1, 4, OP_SUB_C),
            0x92 => (1, 4, OP_SUB_D),
            0x93 => (1, 4, OP_SUB_E),
            0x94 => (1, 4, OP_SUB_H),
            0x95 => (1, 4, OP_SUB_L),
            0x96 => (1, 7, OP_SUB_M),
            0x97 => (1, 4, OP_SUB_A),
            0x98 => (1, 4, OP_SBB_B),
            0x99 => (1, 4, OP_SBB_C),
            0x9A => (1, 4, OP_SBB_D),
            0x9B => (1, 4, OP_SBB_E),
            0x9C => (1, 4, OP_SBB_H),
            0x9D => (1, 4, OP_SBB_L),
            0x9E => (1, 7, OP_SBB_M),
            0x9F => (1, 4, OP_SBB_A),

            0xA0 => (1, 4, OP_ANA_B),
            0xA1 => (1, 4, OP_ANA_C),
            0xA2 => (1, 4, OP_ANA_D),
            0xA3 => (1, 4, OP_ANA_E),
            0xA4 => (1, 4, OP_ANA_H),
            0xA5 => (1, 4, OP_ANA_L),
            0xA6 => (1, 7, OP_ANA_M),
            0xA7 => (1, 4, OP_ANA_A),
            0xA8 => (1, 4, OP_XRA_B),
            0xA9 => (1, 4, OP_XRA_C),
            0xAA => (1, 4, OP_XRA_D),
            0xAB => (1, 4, OP_XRA_E),
            0xAC => (1, 4, OP_XRA_H),
            0xAD => (1, 4, OP_XRA_L),
            0xAE => (1, 7, OP_XRA_M),
            0xAF => (1, 4, OP_XRA_A),

            0xB0 => (1, 4, OP_ORA_B),
            0xB1 => (1, 4, OP_ORA_C),
            0xB2 => (1, 4, OP_ORA_D),
            0xB3 => (1, 4, OP_ORA_E),
            0xB4 => (1, 4, OP_ORA_H),
            0xB5 => (1, 4, OP_ORA_L),
            0xB6 => (1, 7, OP_ORA_M),
            0xB7 => (1, 4, OP_ORA_A),
            0xB8 => (1, 4, OP_CMP_B),
            0xB9 => (1, 4, OP_CMP_C),
            0xBA => (1, 4, OP_CMP_D),
            0xBB => (1, 4, OP_CMP_E),
            0xBC => (1, 4, OP_CMP_H),
            0xBD => (1, 4, OP_CMP_L),
            0xBE => (1, 7, OP_CMP_M),
            0xBF => (1, 4, OP_CMP_A),

            0xC0 => (1, 11, OP_RNZ),
            0xC1 => (1, 10, OP_POP_B),
            0xC2 => (3, 10, OP_JNZ),
            0xC3 | 0xCB => (0, 10, OP_JMP_16), // Always going to jump, no size needed
            0xC4 => (3, 17, OP_CNZ),
            0xC5 => (1, 11, OP_PUSH_B),
            0xC6 => (2, 7, OP_ADI),
            0xC7 => (0, 11, OP_RST_0),
            0xC8 => (1, 11, OP_RZ),
            0xC9 | 0xD9 => (0, 10, OP_RET),
            0xCA => (3, 10, OP_JZ),
            0xCC => (3, 17, OP_CZ),
            0xCD | 0xDD | 0xED | 0xFD => (3, 17, OP_CALL), // Size determined in instr
            0xCE => (2, 2, OP_ACI),
            0xCF => (0, 1, OP_RST_1),

            0xD0 => (1, 11, OP_RNC),
            0xD1 => (1, 10, OP_POP_D),
            0xD2 => (3, 10, OP_JNC),
            0xD3 => (2, 10, OP_OUT),
            0xD4 => (3, 17, OP_CNC),
            0xD5 => (1, 11, OP_PUSH_D),
            0xD6 => (2, 7, OP_SUI),
            0xD7 => (0, 11, OP_RST_2),
            0xD8 => (1, 11, OP_RC),
            0xDA => (3, 10, OP_JC), // Size determined in instruction
            0xDB => (2, 10, OP_IN),
            0xDC => (3, 17, OP_CC),
            0xDE => (2, 7, OP_SBI),
            0xDF => (0, 11, OP_RST_3),

            0xE0 => (1, 11, OP_RPO),
            0xE1 => (1, 10, OP_POP_H),
            0xE2 => (3, 10, OP_JPO),
            0xE3 => (1, 18, OP_XTHL),
            0xE4 => (3, 17, OP_CPO),
            0xE5 => (1, 11, OP_PUSH_H),
            0xE6 => (2, 7, OP_ANI),
            0xE7 => (0, 11, OP_RST_4),
            0xE8 => (1, 11, OP_RPE),
            0xE9 => (0, 5, OP_PCHL),
            0xEA => (3, 10, OP_JPE),
            0xEB => (1, 5, OP_XCHG),
            0xEC => (3, 17, OP_CPE),
            0xEE => (2, 7, OP_XRI),
            0xEF => (0, 11, OP_RST_5),

            0xF0 => (1, 11, OP_RP),
            0xF1 => (1, 10, OP_POP_PSW),
            0xF2 => (3, 10, OP_JP_16),
            0xF3 => (1, 4, OP_DI),
            0xF4 => (3, 17, OP_CP),
            0xF5 => (1, 11, OP_PUSH_PSW),
            0xF6 => (2, 7, OP_ORI),
            0xF7 => (0, 11, OP_RST_6),
            0xF8 => (1, 11, OP_RM),
            0xF9 => (1, 5, OP_SPHL),
            0xFA => (3, 10, OP_JM_16),
            0xFB => (1, 4, OP_EI),
            0xFC => (3, 17, OP_CM),
            0xFE => (2, 7, OP_CPI),
            0xFF => (0, 11, OP_RST_7),
        };

        Instruction {
            opcode,
            size: info.0,
            cycles: info.1,
            text: info.2,
        }
    }
}
