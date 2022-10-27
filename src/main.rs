#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
use egui::Checkbox;
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};
use egui_sdl2_gl as egui_backend;
use sdl2::keyboard::Keycode;
use sdl2::video::SwapInterval;
use std::time::Instant;

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;
use crate::video::Video;
use clap::{App, Arg};
use constants::{CELL_SIZE, DISP_HEIGHT, DISP_WIDTH, WHITE};
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

        match load_rom(&mut cpu, file_to_load.clone(), dims.1) {
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
fn main() -> Result<(), String> {
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

    // Setup our windowing and display driving
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);

    let window = video_subsystem
        .window("Window title", DISP_WIDTH, DISP_HEIGHT)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    let shader_ver = ShaderVersion::Default;
    let (mut painter, mut egui_state) =
        egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Custom(3.0));

    let mut egui_ctx = egui::CtxRef::default();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Gather from the command the rom to use; Clap won't let us skip this but we
    // load INVADERS by default just in case
    let mut rom_file: String = String::from("INVADERS");
    if let Some(f) = matches.value_of("rom") {
        rom_file = String::from(f);
    }

    // Thread status flags
    let cpu_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let cpu_alive_clone = Arc::clone(&cpu_alive);
    let video_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let video_alive_clone = Arc::clone(&video_alive);

    // Actual threaded bits
    let cpu = Arc::new(Mutex::new(Emulator::new(&rom_file)?));
    let cpu_clone = Arc::clone(&cpu);
    let video = Arc::new(Mutex::new(Video::new()));
    let video_clone = Arc::clone(&video);

    // If we are in debug mode, set that now
    if matches.is_present("pause") {
        println!("Setting pause on tick mode; <s> to step; <F1> to toggle; <F2> to kill CPU;");
        cpu_clone.lock().unwrap().cpu.ok_to_step = false; // Will ensure we wait before executing first opcode!
        cpu_clone.lock().unwrap().cpu.single_step_mode = true;
    }

    // Create a thread that will be our running cpu
    // It's just gonna tick like a boss, until it's told not to.
    let cpu_thread_handle = thread::spawn(move || {
        while cpu_alive_clone.load(Ordering::Relaxed) {
            match cpu_clone.lock().unwrap().update() {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to tick: {}", e);
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

    // Create a thread that will be our video processing engine
    // Right now this will just tick a few times and then kill itself.
    let video_thread_handle = thread::spawn(move || {
        while video_alive_clone.load(Ordering::Relaxed) {
            video_clone.lock().unwrap().tick();
            if video_clone.lock().unwrap().tick_count > 5 {
                video_alive_clone.store(false, Ordering::Relaxed);
            }
        }
    });

    // Just some horseshit to display in the window temporarily
    let mut test_str: String = "This is a test string.".to_owned();
    let mut enable_vsync = false;
    let mut quit = false;
    let mut slider = 0.0;

    let start_time = Instant::now();

    let cpu_clone = Arc::clone(&cpu);
    let cpu_alive_clone = Arc::clone(&cpu_alive);

    'running: loop {
        // TODO: Should these be outside the loop?
        // let app_clone = Arc::clone(&cpu);
        // let this_cpu = &app_clone.lock().unwrap().cpu;
        

        // If the cpu is not alive, we should just bail as well.
        if !cpu_alive_clone.load(Ordering::Relaxed) {
            println!("CPU is not alive.  Shutting application down.");
            break 'running;
        }

        // Update details in the windowing subsystem if needs be
        if enable_vsync {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::VSync)
                .unwrap();
        } else {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::Immediate)
                .unwrap();
        }

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());


        // This is the layout of our UI, using egui things
        egui::SidePanel::left("my_left_panel").show(&egui_ctx, |ui| {
            if ui.button("Toggle Pause").clicked() {
                cpu_clone.lock().unwrap().cpu.toggle_single_step_mode()
            }
            if ui.button("Quit").clicked() {
                quit = true;
            }
        });

        // Bottom panel will hold current instructions run history
        egui::TopBottomPanel::bottom("bottom_panel").show(&egui_ctx, |ui| {
            let loop_cpu: &mut CPU = &mut cpu_clone.lock().unwrap().cpu;
            ui.label("Instruction Running Next:");
            ui.label(format!("{} @ {}", loop_cpu.current_instruction, loop_cpu));
        });


        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            let loop_cpu: &mut CPU = &mut cpu_clone.lock().unwrap().cpu;

            ui.label("ROM Display Area");
            ui.separator();
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();

        egui_state.process_output(&window, &egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        // To determine if we need to repaint..
        // if !egui_output.needs_repaint { ... see example ...}

        painter.paint_jobs(None, paint_jobs, &egui_ctx.font_image());
        window.gl_swap_window();
        if let Some(event) = event_pump.wait_event_timeout(5) {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    quit = true;
                    cpu_alive_clone.store(false, Ordering::Relaxed);
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => cpu_clone.lock().unwrap().cpu.toggle_single_step_mode(),
                Event::KeyDown {
                    keycode: Some(Keycode::F2), 
                    ..
                } => cpu_alive.store(false, Ordering::Relaxed),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => cpu_clone.lock().unwrap().cpu.ok_to_step = true, // Setting to false will let it out of the while loop
                _ => {
                    // Process input event
                    egui_state.process_input(&window, event, &mut painter);
                }
            }
        }

        // Sleep a bit
        //thread::sleep(Duration::from_millis(1));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        if quit {
            break;
        }
    }

    cpu_thread_handle.join().unwrap();
    video_thread_handle.join().unwrap();

    // lib::go()?;
    Ok(())
}
