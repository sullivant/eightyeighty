mod cpu;
pub use cpu::Cpu;

pub fn go() {
    let mut cpu = Cpu::new();
    cpu.set_disassemble(true);
    let rom_file = "./resources/roms/invaders.h".to_string();

    // TODO: This will eventually end up really loading the rom
    match cpu.load_rom(rom_file.clone()) {
        Ok(_) => println!("Loaded rom file: {}", rom_file),
        Err(err) => {
            panic!("Unable to load rom file: {}", err);
        }
    }
}
