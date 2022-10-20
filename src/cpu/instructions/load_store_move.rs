use crate::cpu::{Registers, CPU};

/// This contains any instructions of the LOAD / STORE / MOVE category
/// that need to be implemented within the CPU

impl CPU {
    // LXI (target pair), D16
    pub fn op_lxi(&mut self, target: Registers, dl: u8, dh: u8) -> Result<(), String> {
        match target {
            Registers::BC => {
                self.b = dh;
                self.e = dl;
                Ok(())
            }
            Registers::DE => {
                self.d = dh;
                self.e = dl;
                Ok(())
            }
            Registers::HL => {
                self.h = dh;
                self.l = dl;
                Ok(())
            }
            Registers::SP => {
                self.sp = u16::from(dh) << 8 | u16::from(dl);
                Ok(())
            }
            _ => Err(format!(
                "Register {} is NOT IMPLEMENTED in OP_LXI, Cannot Execute",
                target
            )),
        }
    }
}
