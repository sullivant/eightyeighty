#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod constants;
mod cpu;

use crate::cpu::CPU;
use clap::{App, Arg};
use termion::event::Key;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

extern crate termion;

use termion::raw::IntoRawMode;
use termion::async_stdin;
use std::io::{Read, Write, stdout};
use std::time::Duration;



// Our emulator contains a few key components:
// * A CPU which itself contains: Memory, Instructions, Flags, Registers, and the ability to cycle/tick.
// * A windowing environment which will display and provide interrupts for the video section of memory to
// properly operate.
// * I/O - to cover keyboard input, joysticks, sound, and other stuff.

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
        let file_to_load = format!("./resources/roms/{}.COM", rom_file);
        let mut dims: (usize, usize) = (0, 0);

        match cpu.load_rom(file_to_load.clone(), dims.1) {
            Ok(i) => {
                dims = i;
            }
            Err(err) => {
                return Err(format!("Unable to load rom file {}: {}", file_to_load, err));
            }
        }

        println!(
            "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
            file_to_load,
            dims.0,
            dims.1 - 1
        );

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

    // This will be called via the thread, loaded below in go() somewhere...
    fn update(&mut self) -> Result<(), String> {
        // Tick the cpu
        self.cpu.tick()
    }
}

/// go()
///
/// The initial starting point of the application.  This processes parameters,
/// passed from the command line and tries to load a ROM and begin execution.
///
/// # Errors
/// Will return an 'Err' if the filename for the rom passed on the command line does not exist.
///
/// # Panics
/// Will panic if necessary, sending up any of the aformentioned errors.
#[allow(clippy::too_many_lines)]
pub fn go() -> Result<(), String> {
    let mut stdin = async_stdin().bytes();

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

    let cpu_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    // let cpu_alive_clone = Arc::clone(&cpu_alive);

    // Gather from the command the rom to use; Clap won't let us skip this but we
    // load INVADERS by default just in case
    let mut rom_file: String = String::from("INVADERS");
    if let Some(f) = matches.value_of("rom") {
        rom_file = String::from(f);
    }

    let app = Arc::new(Mutex::new(Emulator::new(&rom_file)?));
    let app_clone = Arc::clone(&app);

    // Create a thread that will be our running cpu
    // It's just gonna tick like a boss, until it's told not to.
    let handle = thread::spawn(move || {
        while cpu_alive.load(Ordering::Relaxed) {
            match app.lock().unwrap().update() {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to tick: {}", e);
                    break;
                }
            }
        }

        println!(
            "Shutting down. Final CPU state:\n{}",
            app.lock().unwrap().cpu
        );
    });

    loop {
        let b = stdin.next();

        // if let Some(Ok(b'q')) = b {
        //     break;
        // }

        match b {
            Some(Ok(b'q')) => {break},
            Some(Ok(b's')) => {app_clone.lock().unwrap().cpu.single_step = false},
            _ => break,
        }
    }

    handle.join().unwrap();

    Ok(())
}
