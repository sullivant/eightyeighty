#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod constants;
mod cpu;

use crate::cpu::CPU;
use clap::{App, Arg};
use constants::{CELL_SIZE, DISP_WIDTH, DISP_HEIGHT, WHITE};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

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

    // Some basic stuff, to get our event pump
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(
            "8080",
            (DISP_WIDTH * CELL_SIZE).into(),
            (DISP_HEIGHT * CELL_SIZE).into(),
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    // Reset to start
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.clear();
    canvas.present();

    // Gather from the command the rom to use; Clap won't let us skip this but we
    // load INVADERS by default just in case
    let mut rom_file: String = String::from("INVADERS");
    if let Some(f) = matches.value_of("rom") {
        rom_file = String::from(f);
    }
    
    
    let cpu_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let cpu_alive_clone = Arc::clone(&cpu_alive);
    
    let app = Arc::new(Mutex::new(Emulator::new(&rom_file)?));
    let app_clone = Arc::clone(&app);

    // If we are in debug mode, set that now
    if matches.is_present("pause") {
        println!("Setting pause on tick mode; <s> to step; <F1> to toggle; <F2> to kill CPU;");
        app_clone.lock().unwrap().cpu.ok_to_step = false; // Will ensure we wait before executing first opcode!
        app_clone.lock().unwrap().cpu.single_step_mode = true;
    }

    // Create a thread that will be our running cpu
    // It's just gonna tick like a boss, until it's told not to.
    let handle = thread::spawn(move || {
        while cpu_alive_clone.load(Ordering::Relaxed) {
            match app_clone.lock().unwrap().update() {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to tick: {}", e);
                    break;
                }
            }
        }

        println!(
            "Shutting down. Final CPU state:\n{}",
            app_clone.lock().unwrap().cpu
        );
    });

    // The main application loop that will handle the event pump
    'running: loop {
        let app_clone = Arc::clone(&app);
        let cpu_alive_clone = Arc::clone(&cpu_alive);

        // If the cpu is not alive, we should just bail as well.
        if !cpu_alive_clone.load(Ordering::Relaxed) {
            println!("CPU is not alive.  Shutting application down.");
            break 'running;
        }

        // Hit up the event pump
        for event in event_pump.poll_iter() {
            // Read the keyboard
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    // Tell the CPU to stop
                    cpu_alive_clone.store(false, Ordering::Relaxed);
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => app_clone.lock().unwrap().cpu.toggle_single_step_mode(),
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => cpu_alive.store(false, Ordering::Relaxed),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => app_clone.lock().unwrap().cpu.ok_to_step = true, // Setting to false will let it out of the while loop
                _ => (),
            };
        }

        // Clear the screen
        canvas.clear();

        // Not drawing shit right now...
        // To Draw: 
        // DISASM of entire loaded rom
        // VRAM (Obviously)
        // CPU Info (CPU has print format)
        // Console output?

        // Present the updated screen
        canvas.set_draw_color(WHITE);
        canvas.present();
        
        // Sleep a bit
        //thread::sleep(Duration::from_millis(1));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }


    handle.join().unwrap();

    Ok(())
}
