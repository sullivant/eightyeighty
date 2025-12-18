use emulator::{self, Emulator};

use crate::{
    emulator::cpu::StepResult,
};

fn main() -> Result<(), String> {
    let mut emu: Emulator = Emulator::new();

    const ROM_TST: &[u8] = &[0x3E, 0x42, 0x76];

    emu.load_rom(ROM_TST)?;


    let result: StepResult = emu.step()?;
    println!("{}", result);

    Ok(())
}
