use crate::{cpu::ProgramCounter, Cpu};

/// This file contains the functions needed for the "General / Control" opcodes
///

impl Cpu {
    // OUT D8
    // Would send the contents of accumulator to the device sent
    // as the data portion of this command
    // TODO: If data out is needed, this needs to be finished
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn op_out(&self, _data: u8) -> ProgramCounter {
        ProgramCounter::Two
    }
    // ProgramCounter is incremented and then the CPU enters a
    // STOPPED state and no further activity takes place until
    // an interrupt occurrs
    pub fn op_hlt(&mut self) -> ProgramCounter {
        self.set_nop(true);
        ProgramCounter::Next
    }

}
