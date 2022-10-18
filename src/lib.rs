#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod constants;
mod cpu;
mod disassembler;

pub use crate::constants::*;
pub use crate::cpu::common::*;
pub use crate::cpu::Cpu;

use clap::{App, Arg};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
//use sdl2::rect::{Point, Rect};
use sdl2::rect::Point;
use std::i64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Specific color stuff for the UI
const WHITE: Color = Color::RGB(255, 255, 255);
const BLACK: Color = Color::RGB(0, 0, 0);
//const RED: Color = Color::RGB(255, 0, 0);
//const GREEN: Color = Color::RGB(0, 255, 0);

#[derive(Clone)]
pub struct Emu {
    cpu: Cpu,
    last_pc: usize,
    pause_on_tick: bool,
    single_tick: bool,
    pause_on_count: usize,
}

impl Emu {
    fn new(rom_file: &str) -> Result<Emu, String> {
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
        Ok(Emu {
            cpu,
            last_pc: 0,
            pause_on_tick: false,
            single_tick: false,
            pause_on_count: 0,
        })
    }

    fn set_single_tick(&mut self, n: bool) {
        self.single_tick = n;
    }

    fn set_pause_on_tick(&mut self, v: bool) {
        self.pause_on_tick = v;
    }

    fn toggle_pause_on_tick(&mut self) {
        self.pause_on_tick = !self.pause_on_tick;
    }

    fn set_pause_on_count(&mut self, v: usize) {
        self.pause_on_count = v;
    }

    fn update(&mut self) -> Result<(), String> {
        if self.cpu.cycle_count > 0 && self.cpu.cycle_count == self.pause_on_count {
            self.pause_on_tick = true;
        }

        // If we are not in pause_on_tick mode, tick away
        if self.pause_on_tick {
            // We want to tick only when tick_once is true (Space key sets this)
            if self.single_tick {
                // Tick the cpu
                match self.cpu.tick() {
                    Ok(n) => {
                        self.last_pc = n;
                    }
                    Err(e) => {
                        return Err(format!("Unable to single tick {}", e));
                    }
                }
                self.single_tick = false;
            }
        } else {
            // Tick the cpu
            match self.cpu.tick() {
                Ok(n) => {
                    self.last_pc = n;
                }
                Err(e) => {
                    return Err(e);
                }
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

    // Create a window.
    let sdl_context = sdl2::init()?;
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

    // Canvas stuff
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    // Reset to start
    canvas.clear();
    canvas.present();

    let cpu_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let cpu_alive_clone = Arc::clone(&cpu_alive);

    // Build our application and include the CLI options if necessary

    // Gather from the command the rom to use; Clap won't let us skip this but we
    // load INVADERS by default just in case
    let mut rom_file: String = String::from("INVADERS");
    if let Some(f) = matches.value_of("rom") {
        rom_file = String::from(f);
    }

    let app = Arc::new(Mutex::new(Emu::new(&rom_file)?));
    let app_clone = Arc::clone(&app);

    // If we are in debug mode, set that now
    if matches.is_present("pause") {
        println!("Setting pause on tick mode; <SPACEBAR> to step; <F1> to toggle;");
        app_clone.lock().unwrap().set_pause_on_tick(true);
    }

    if let Some(c) = matches.value_of("count") {
        if let Ok(r) = i64::from_str_radix(c, 16) {
            println!("Pause will happen at cycle count: {:#06X}", r);
            app_clone.lock().unwrap().set_pause_on_count(r as usize);
        }
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

    // Main loop
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
                    cpu_alive.store(false, Ordering::Relaxed);
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => app_clone.lock().unwrap().toggle_pause_on_tick(),
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => cpu_alive.store(false, Ordering::Relaxed),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => app_clone.lock().unwrap().set_single_tick(true),
                _ => (),
            };
        }

        // Clear the screen
        canvas.clear();

        // Draw the graphics portion of memory (TODO)
        canvas.set_draw_color(BLACK);
        // Bottom border of EMU display area
        canvas.draw_line(
            Point::new(0, (EMU_HEIGHT * CELL_SIZE).into()),
            Point::new(
                (EMU_WIDTH * CELL_SIZE).into(),
                (EMU_HEIGHT * CELL_SIZE).into(),
            ),
        )?;
        // Right border of EMU display area
        canvas.draw_line(
            Point::new((EMU_WIDTH * CELL_SIZE).into(), 0),
            Point::new(
                (EMU_WIDTH * CELL_SIZE).into(),
                (EMU_HEIGHT * CELL_SIZE).into(),
            ),
        )?;

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
