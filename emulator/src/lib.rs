#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;
mod utils;

use crate::cpu::CPU;
use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use wasm_bindgen::prelude::*;


#[derive(Clone)]
pub struct Emulator {
    cpu: CPU,
}

impl Emulator {
    fn new(rom_file: &str) -> Result<Emulator, String> {
        println!("Creating new Emulator Object");

        // Generate our CPU
        let mut cpu = CPU::new();
        cpu.disassemble(true);

        // The list of rom files to load for this particular collection/game
        let file_to_load = format!("./resources/roms/{rom_file}.COM");
        let mut dims: (usize, usize) = (0, 0);

        match load_rom(&mut cpu, file_to_load.clone(), dims.1) {
            Ok(i) => {
                dims = i;
            }
            Err(err) => {
                return Err(format!("Unable to load rom file {file_to_load}: {err}"));
            }
        }

        println!(
            "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
            file_to_load,
            dims.0,
            dims.1 - 1
        );

        // TODO: Remove when done tinkering
        //println!("{}",cpu.memory());

        // For testing the odd CPUDIAG ROM
        // if file_to_load.eq("./resources/roms/TST8080.COM") {
        //     println!("TS8080 loaded, making some debug changes");
        //     // First, make a jump to 0x0100
        //     cpu.memory[0] = 0xC3;
        //     cpu.memory[1] = 0x00;
        //     cpu.memory[2] = 0x01;
        // }

        // Return a good version of the app object
        Ok(Emulator { cpu })
    }

    /// Performs an actual "tick" of the CPU for a single instruction.
    /// 
    /// This will be called via the thread, loaded below in go() somewhere...
    fn update(&mut self) -> Result<(), String> {
        // Tick the cpu
        self.cpu.tick()
    }
}

/// Load the ROM file into memory, starting at ``start_index``
/// Returns a tuple containing the index we started at and where we
/// actually finished at.
///
/// # Errors
/// Will return a standard io Error if necessary
/// # Panics
/// If the error happens, this will cause the function to panic
pub fn load_rom(
    cpu: &mut CPU,
    file: String,
    start_index: usize,
) -> Result<(usize, usize), std::io::Error> {
    let rom = File::open(file)?;
    let mut last_idx: usize = 0;
    for (i, b) in rom.bytes().enumerate() {
        cpu.memory().write(start_index + i, b.unwrap()).unwrap();
        last_idx = i;
    }
    Ok((start_index, start_index + last_idx + 1))
}



#[allow(clippy::too_many_lines)]
pub fn go() -> Result<(), String> {
    // Get some cli options from CLAP
    let matches = App::new("EightyEighty")
        .version("1.0")
        .author("Thomas Sullivan <sullivan.t@gmail.com>")
        .about("An 8080 emulator")
        .arg(Arg::from_usage(
            "-p, --pause... 'initiates single step (pause on tick) mode'",
        ))
        .arg(Arg::from_usage(
            "-c, --count=[COUNT] 'pauses and initiates single step mode on program count <count>'",
        ))
        .args_from_usage("<rom> 'The rom file to load and execute'")
        .get_matches();


    // Gather from the command the rom to use; Clap won't let us skip this but we
    // load INVADERS by default just in case
    let mut rom_file: String = String::from("INVADERS");
    if let Some(f) = matches.value_of("rom") {
        rom_file = String::from(f);
    }

    // Thread status flags
    let cpu_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true)); // Used to bail out of the threads if needed

    // Actual threaded bits
    let cpu = Arc::new(Mutex::new(Emulator::new(&rom_file)?)); // A threadable version of our emulator
    let cpu_clone = Arc::clone(&cpu); // Used to tickle various settings

    // If we are in debug mode, set that now, before we start
    if matches.is_present("pause") {
        println!("Setting pause on tick mode; <s> to step; <F1> to toggle; <F2> to kill CPU;");
        cpu_clone.lock().unwrap().cpu.ok_to_step = false; // Will ensure we wait before executing first opcode!
        cpu_clone.lock().unwrap().cpu.single_step_mode = true;
    }

    // Create a thread that will be our running cpu
    // It's just gonna tick like a boss, until it's told not to.
    let cpu_thread_handle = thread::spawn(move || {
        while cpu_alive.load(Ordering::Relaxed) {
            match cpu_clone.lock().unwrap().update() {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to tick: {e}");
                    break;
                }
            }

            // TODO: Make some form of cycle based speed regulation
        }

        println!(
            "Shutting down. Final CPU state:\n{}",
            cpu_clone.lock().unwrap().cpu
        );
    });

    cpu_thread_handle.join().unwrap();
    // lib::go()?;
    Ok(())
}



#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, from wasm");
}

