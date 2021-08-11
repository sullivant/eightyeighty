mod cpu;
mod disassembler;
pub use cpu::Cpu;

pub fn go() {
    let mut cpu = Cpu::new();
    cpu.set_disassemble(true);
    cpu.set_nop(true);

    // The list of rom files to load for this particular collection/game
    let rom_files: [String; 4] = [
        String::from("./resources/roms/invaders.h"),
        String::from("./resources/roms/invaders.g"),
        String::from("./resources/roms/invaders.f"),
        String::from("./resources/roms/invaders.e"),
    ];
    let mut dims: (usize, usize) = (0, 0);

    for f in rom_files {
        match cpu.load_rom(f.clone(), dims.1) {
            Ok(i) => {
                dims = i;
            }
            Err(err) => {
                panic!("Unable to load rom file {}: {}", f, err);
            }
        }
        println!(
            "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
            f,
            dims.0,
            dims.1 - 1
        );
    }

    if cpu.disassemble {
        println!("PC\tIns  S\t[l,h]\t\tData(lo,hi)\tCommand");
    }

    for _ in 0..0x10 {
        cpu.tick();
    }
}
