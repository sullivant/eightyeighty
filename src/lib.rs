#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod constants;
mod cpu;
mod disassembler;

pub use crate::constants::*;
pub use crate::cpu::common::*;
pub use crate::cpu::Cpu;

use clap::{App, Arg};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;


#[derive(Clone)]
pub struct Emulator {
    cpu: Cpu,
    last_pc: usize
}

impl Emulator {
    fn new(rom_file: &str) -> Result<Emulator, String> {
        println!("Creating new Emu Object");

        // Generate our CPU
        let mut cpu = Cpu::new();
        cpu.set_disassemble(true);
        //cpu.set_nop(true);

        // The list of rom files to load for this particular collection/game
        let file_to_load = format!("./resources/roms/{}.COM", rom_file);
        let mut dims: (usize, usize) = (0, 0);

        match cpu.load_rom(file_to_load.clone(), dims.1) {
            Ok(i) => {
                dims = i;
            }
            Err(err) => {
                return Err(format!("Unable to load rom file {}: {}", file_to_load, err));
                //panic!("Unable to load rom file {}: {}", file_to_load, err);
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
        Ok(Emulator {
            cpu,
            last_pc: 0
        })
    }


    fn update(&mut self) -> Result<(), String> {
        // Tick the cpu
        match self.cpu.tick() {
            Ok(n) => {
                self.last_pc = n;
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
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
    // let app_clone = Arc::clone(&app);

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


    handle.join().unwrap();

    Ok(())
}

// // Does what it says on the tin.
// fn add_display_text(canvas: &mut sdl2::render::WindowCanvas, to_display: &str, x: i32, y: i32) {
//     let texture_creator = canvas.texture_creator();
//     let ttf_context = sdl2::ttf::init().unwrap();
//     let font = ttf_context
//         .load_font("./resources/fonts/CamingoCode-Regular.ttf", 16)
//         //.load_font("./resources/fonts/OpenSans-Regular.ttf", 16)
//         //.load_font("./resources/fonts/xkcd-script.ttf", 20)
//         .unwrap();

//     let surface = font.render(to_display).solid(BLACK).unwrap();
//     let texture = texture_creator
//         .create_texture_from_surface(&surface)
//         .unwrap();

//     canvas
//         .copy(
//             &texture,
//             None,
//             Some(Rect::new(x, y, surface.width(), surface.height())),
//         )
//         .unwrap();
// }
