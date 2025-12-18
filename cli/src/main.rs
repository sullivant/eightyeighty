use std::io::{Write, stdin, stdout};

use emulator::{self, Emulator, cpu::CPU};

use crate::{
    emulator::cpu::StepResult,
};

fn main() -> Result<(), String> {
    const ROM_TST: &[u8] = &[0x3E, 0x42, 0x76];

    println!("Creating emulator...");
    let mut emu: Emulator = Emulator::new();

    println!("Loading rom...");
    emu.load_rom(ROM_TST)?;


    println!("Starting REPL...");

    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input {
            "quit" | "exit" => break,
            "step" => step(&mut emu.cpu),
            cmd if cmd.starts_with("run ") => run(&mut emu.cpu, cmd),
            cmd if cmd.starts_with("mem ") => mem(&mut emu.cpu, cmd),
            "regs" => regs(&emu.cpu),
            "pc" => println!("PC = {:04X}", emu.cpu.pc),
            _ => println!("Unknown command"),
        }

    }

    Ok(())
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

fn run(cpu: &mut CPU, cmd: &str) {
    let parts: Vec<_> = cmd.split_whitespace().collect();
    let target_cycles: u64 = match parts.get(1).and_then(|s| s.parse().ok()) {
        Some(v) => v,
        None => {
            println!("Usage: run <cycles>");
            return;
        }
    };

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
