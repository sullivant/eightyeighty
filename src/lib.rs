mod cpu;
mod disassembler;
pub use cpu::Cpu;

pub const OPCODE_SIZE: usize = 1;
pub const FLAG_CARRY: u8 = 0b0001_0000; //4
pub const FLAG_ZERO: u8 = 0b0000_1000; //3
pub const FLAG_SIGN: u8 = 0b0000_0100; //2
pub const FLAG_PARITY: u8 = 0b0000_0010; //1
pub const FLAG_AUXCARRY: u8 = 0b0000_0000; //0

pub fn go() {
    let mut cpu = Cpu::new();
    cpu.set_disassemble(true);
    cpu.set_nop(true);

    // The list of rom files to load for this particular collection/game
    let rom_files: [String; 1] = [
        //String::from("./resources/roms/TST8080.COM"),
        String::from("./resources/roms/INVADERS.COM"),
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
        println!("CYCLE:PC\tIns  S\t[l,h]\t\tczspa\tData(lo,hi)\tB\tCommand");
    }

    let mut i = 0;
    loop {
        match cpu.tick(i) {
            Ok(_) => {}
            Err(err) => {
                panic!("Unable to tick: {}", err);
            }
        }
        i += 1;
    }
}
