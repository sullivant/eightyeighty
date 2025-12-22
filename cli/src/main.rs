use std::io::{Write, stdin, stdout};

use rustyline::{error::ReadlineError, history};
use rustyline::DefaultEditor;

use emulator::{self, Emulator, cpu::CPU};

// A simple test rom with a few instructions
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

                rl.add_history_entry(line);

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

fn setup_emu() -> Result<Emulator, String> {
    // Put this in a setup fn
    println!("Creating emulator...");
    let mut emu: Emulator = Emulator::new();
    println!("Inserting ROM and loading...");
    emu.load_rom(ROM_TST.to_vec())?;

    return Ok(emu);
}


fn handle_command(emu: &mut Emulator, line: &str) -> bool {

    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts.as_slice() {
        ["quit"] | ["exit"] => return false,

        ["step"] => {
            step(&mut emu.cpu);
        },

        ["run", cycles] => {
            if let Ok(c) = cycles.parse::<u64>() {
                run(&mut emu.cpu, c);
            }
        },

        ["regs"] => regs(&emu.cpu),

        // Will resend the line, to be properly parsed in the mem fn.
        ["mem", _, _] => mem(&mut emu.cpu, line),

        ["pc"] => println!("PC = {:04X}", emu.cpu.pc),

        ["reset"] => {
            println!("Resetting Emulator");
            if let Err(_) = emu.reset() {
                println!("Error in resetting!");
                return false;
            }
        },

        _ => println!("Unknown command: {}", line),
    }

    true
}


fn regs(cpu: &CPU) {
    println!(
        "A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X}",
        cpu.a, cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, cpu.sp, cpu.pc
    );
}

fn mem(cpu: &CPU, cmd: &str) {
    let parts: Vec<_> = cmd.split_whitespace().collect();
    if parts.len() != 3 {
        println!("Usage: mem <addr> <len>");
        return;
    }

    let addr = usize::from_str_radix(parts[1], 16).unwrap_or(0);
    let len = parts[2].parse::<usize>().unwrap_or(0);

    for i in 0..len {
        let v = cpu.memory.read(addr + i).unwrap_or(0);
        println!("{:04X}: {:02X}", addr + i, v);
    }
}


fn step(cpu: &mut CPU) {
    match cpu.step() {
        Ok(result) => {
            println!(
                "{:04X}: {:02X}  {:<10}  +{} cycles",
                result.pc,
                result.opcode,
                result.mnemonic,
                result.cycles
            );
        }
        Err(e) => println!("Error: {e}"),
    }
}

fn run(cpu: &mut CPU, target_cycles: u64) {
    let mut cycles_run = 0u64;
    let mut instr_count = 0;

    while cycles_run < target_cycles {
        match cpu.step() {
            Ok(result) => {
                cycles_run += result.cycles as u64;
                instr_count += 1;

                // Stop on HLT
                if result.opcode == 0x76 {
                    println!("HLT encountered");
                    break;
                }
            }
            Err(e) => {
                println!("CPU error: {e}");
                break;
            }
        }
    }

    println!(
        "Ran {} instructions, {} cycles (target {})",
        instr_count, cycles_run, target_cycles
    );
}
