use std::io::{self};
use std::fs;

use emulator::{RunState, RunStopReason};
use rustyline::{error::ReadlineError};
use rustyline::DefaultEditor;

use emulator::{self, Emulator, cpu::CPU, bus::Bus};

// A simple test rom with a few instructions to load at the start
const ROM_TST: &[u8] = &[0x3E, 0x42, 0x76];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let prompt = "8080> ";

    // Store REPL history
    let history_path = ".history";
    let _ = rl.load_history(history_path);

    let mut emu: Emulator = setup_emu()?;

    println!("Starting REPL...");
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line)?;

                if !handle_command(&mut emu, line) {
                    break;
                }
            }

            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }

            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(history_path);
    Ok(())
}

/// Will create the emulator machine, and insert the "default" ROM
fn setup_emu() -> Result<Emulator, String> {
    // Put this in a setup fn
    println!("Creating emulator...");
    let mut emu: Emulator = Emulator::new();
    println!("Inserting ROM and loading...");
    emu.load_rom(ROM_TST.to_vec())?;

    Ok(emu)
}

/// Actually handles processing the REPL command
fn handle_command(emu: &mut Emulator, line: &str) -> bool {

    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts.as_slice() {
        ["quit"] | ["exit"] => return false,

        ["step"] => {
            step(emu);
        },

        ["run", cycles] => {
            if let Ok(c) = cycles.parse::<u64>() {
                run(emu, c);
            }
        },

        ["regs"] => regs(&emu.cpu),
        ["emu"] => emu_state(emu),

        // Will resend the line, to be properly parsed in the mem fn.
        ["mem", _, _] => mem(&emu.bus, line),

        ["rom"] => print_rom(emu),

        ["pc"] => println!("PC = {:04X}", emu.cpu.pc),

        ["insert", rom_name] => {
            let file = if rom_name.ends_with(".rom") {
                rom_name.to_string()
            } else {
                format!("{}.rom", rom_name)
            };

            let path = format!("resources/roms/{}", file);
            println!("Inserting ROM: {}", path);

            // If it loads from the file, stuff it into the Emulator
            match load_rom_file(&path) {
                Ok(bytes) => {
                    emu.insert_rom(bytes);
                }
                Err(e) => {
                    println!("File error: {}", e);
                }
            }
        },

        ["remove"] => {
            println!("Removing ROM from Emulator");
            emu.remove_rom();
        },

        ["reset"] => {
            println!("Resetting Emulator");
            if let Err(e) = emu.reset() {
                println!("Error in resetting: {}",e);
                return false;
            }
        },

        _ => println!("Unknown command: {}", line),
    }

    true
}


/// Displays registers
fn regs(cpu: &CPU) {
    println!(
        "A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X}",
        cpu.a, cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, cpu.sp, cpu.pc
    );
}

/// Displays emulator state
fn emu_state(emu: &mut Emulator) {
    match emu.run_state() {
        RunState::Running => { println!("State: Running");},
        RunState::Stopped => { println!("State: Stopped");},
    }
}

/// Displays a portion of memory
fn mem(bus: &Bus, cmd: &str) {
    let parts: Vec<_> = cmd.split_whitespace().collect();
    if parts.len() != 3 {
        println!("Usage: mem <addr> <len>");
        return;
    }

    let addr = usize::from_str_radix(parts[1], 16).unwrap_or(0);
    let len = parts[2].parse::<usize>().unwrap_or(0);
    let bytes_per_line = 16;

    for line_start in (0..len).step_by(bytes_per_line) {
        // Print the address at start of line
        print!("{:04X}:  ", addr + line_start);

        // Print hex bytes for this line
        for i in 0..bytes_per_line {
            let idx = line_start + i;
            if idx < len {
                let byte = bus.read(addr + idx);
                print!("{:02X} ", byte);
            } else {
                // Padding for incomplete line
                print!("   ");
            }
        }

        // Print ASCII characters for this line
        print!(" ");

        for i in 0..bytes_per_line {
            let idx = line_start + i;
            if idx < len {
                let byte = bus.read(addr + idx);
                // Show printable ASCII or '.' for non-printable
                let ch = if byte.is_ascii_graphic() || byte == b' ' {
                    byte as char
                } else {
                    '.'
                };
                print!("{}", ch);
            } else {
                // Padding for incomplete line
                print!(" ");
            }
        }

        println!();
    }
}

/// Prints the currently loaded ROM in case one is curious
fn print_rom(emu: &Emulator) {
    match emu.rom() {
        Some(rom) => {
            for (addr, byte) in rom.iter().enumerate() {
                println!("{:04X}: {:02X}", addr, byte);
            }
        }
        None => {
            println!("No ROM loaded.");
        }
    }
}

/// Issues a single step command and shows what happened and how many cycles it took
fn step(emu: &mut Emulator) {
    match emu.step() {
        Some(result) => {
            println!(
                "{:04X}: {:02X}  {:<10}  +{} cycles",
                result.pc,
                result.opcode,
                result.mnemonic,
                result.cycles
            );
        }
        _ => (),
    }
}

/// Runs for a certain number of cycles; handled by the emulator
fn run(emu: &mut Emulator, target_cycles: u64) {
    match emu.run_blocking(Some(target_cycles)) {
        RunStopReason::CycleBudgetExhausted => { println!("Stopped: Cycle budget exhausted.");},
        RunStopReason::Halted => { println!("Stopped: Halted.");},
        _ => { println!("Stopped: Unknown reason.");}
    }
    
}

/// Just loads provided filepath into a vec.
fn load_rom_file(path: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
}