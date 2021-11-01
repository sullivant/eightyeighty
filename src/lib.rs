mod cpu;
mod disassembler;

use clap::{App, Arg};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::i64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub use cpu::Cpu;
pub const OPCODE_SIZE: usize = 1;

// S - Sign Flag
// Z - Zero Flag
// 0 - Not used, always zero
// A - also called AC, Auxiliary Carry Flag
// 0 - Not used, always zero
// P - Parity Flag
// 1 - Not used, always one
// C - Carry Flag
pub const FLAG_SIGN: u8 = 0b1000_0000;
pub const FLAG_ZERO: u8 = 0b0100_0000;
pub const FLAG_AUXCARRY: u8 = 0b0001_0000;
pub const FLAG_PARITY: u8 = 0b0000_0100;
pub const FLAG_CARRY: u8 = 0b0000_0001;

// Window and display concerns
pub const DISP_WIDTH: u16 = 640; // Overall width/height
pub const DISP_HEIGHT: u16 = 480;
pub const EMU_WIDTH: u16 = 224; // Emulator display area width/height
pub const EMU_HEIGHT: u16 = 256;
pub const CELL_SIZE: u16 = 2; // The size of a "cell" or pixel
const LINE_SPACE: u16 = 20; // Space between lines of text
const WHITE: Color = Color::RGB(255, 255, 255);
const BLACK: Color = Color::RGB(0, 0, 0);
//const RED: Color = Color::RGB(255, 0, 0);
//const GREEN: Color = Color::RGB(0, 255, 0);

#[derive(Clone)]
pub struct Emu {
    dt: std::time::Duration,
    cpu: Cpu,
    last_msg: String, // Contains last disassembler message
    last_pc: usize,
    pause_on_tick: bool,
    single_tick: bool,
    pause_on_count: usize,
}

impl Emu {
    fn new(rom_file: String) -> Result<Emu, String> {
        println!("Creating new Emu Object");
        let dt = std::time::Duration::new(0, 0);

        // Generate our CPU
        let mut cpu = Cpu::new();
        cpu.set_disassemble(true);
        cpu.set_nop(true);

        // The list of rom files to load for this particular collection/game
        let file_to_load = format!("./resources/roms/{}.COM", rom_file);
        let mut dims: (usize, usize) = (0, 0);

        match cpu.load_rom(file_to_load.clone(), dims.1) {
            Ok(i) => {
                dims = i;
            }
            Err(err) => {
                panic!("Unable to load rom file {}: {}", file_to_load, err);
            }
        }

        println!(
            "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
            file_to_load,
            dims.0,
            dims.1 - 1
        );

        // Return a good version of the app object
        Ok(Emu {
            dt,
            cpu,
            last_msg: "N/A".to_string(),
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

    // For the debugger and such will go through needed items and prep them for display
    // on the output window located in the canvas reference
    fn create_display_texts(&mut self, canvas: &mut sdl2::render::WindowCanvas) {
        // Cycle count
        add_display_text(
            canvas,
            &format!("Cycle:{:#06X}", self.cpu.cycle_count),
            0,
            (EMU_HEIGHT * CELL_SIZE).into(),
        );

        // Command issued
        add_display_text(
            canvas,
            disassembler::HEADER,
            0,
            ((EMU_HEIGHT * CELL_SIZE) + (LINE_SPACE * 3)) as i32,
        );
        add_display_text(
            canvas,
            &self.last_msg.to_string(),
            0,
            ((EMU_HEIGHT * CELL_SIZE) + (LINE_SPACE * 4)) as i32,
        );

        // Flags
        add_display_text(
            canvas,
            &"SZ0A0P1C".to_string(),
            ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
            0,
        );
        add_display_text(
            canvas,
            &format!("{:08b}", self.cpu.get_flags()),
            ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
            (LINE_SPACE) as i32,
        );

        // Stack
        add_display_text(
            canvas,
            &"---Stack---".to_string(),
            ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
            (LINE_SPACE * 2) as i32,
        );

        for i in 0..3 {
            add_display_text(
                canvas,
                &format!(
                    "$[{:04X}] = {:02X}",
                    self.cpu.sp + i,
                    self.cpu.memory[(self.cpu.sp + i) as usize]
                ),
                ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
                (LINE_SPACE * (i + 3)) as i32,
            );
        }

        // Registers
        add_display_text(
            canvas,
            &"---Registers---".to_string(),
            ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
            (LINE_SPACE * 6) as i32,
        );

        for (i, r) in ["A", "B", "C", "D", "E", "H", "L"].iter().enumerate() {
            let val = match *r {
                "A" => self.cpu.a,
                "B" => self.cpu.b,
                "C" => self.cpu.c,
                "D" => self.cpu.d,
                "E" => self.cpu.e,
                "H" => self.cpu.h,
                "L" => self.cpu.l,
                _ => 0,
            };
            add_display_text(
                canvas,
                &format!("{} = {:04X}", r, val),
                ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE) as i32,
                (LINE_SPACE * (i as u16 + 7)) as i32,
            );
        }

        // Register Pairs
        for (i, r) in [cpu::Registers::BC, cpu::Registers::DE, cpu::Registers::HL]
            .iter()
            .enumerate()
        {
            let val = self.cpu.get_register_pair(*r);
            add_display_text(
                canvas,
                &format!("Pair: {} {:04X}", r, val),
                ((EMU_WIDTH * CELL_SIZE) + CELL_SIZE * 50) as i32,
                (LINE_SPACE * (i as u16 + 7)) as i32,
            );
        }
    }

    fn update(&mut self) -> Result<(), String> {
        let mut tick_happened: bool = false;

        if self.cpu.cycle_count > 0 && self.cpu.cycle_count == self.pause_on_count {
            self.pause_on_tick = true;
        }

        // If we are not in pause_on_tick mode, tick away
        if !self.pause_on_tick {
            // Tick the cpu
            match self.cpu.tick() {
                Ok(n) => {
                    tick_happened = true;
                    self.last_pc = n;
                }
                Err(e) => {
                    panic!("Unable to tick {}", e);
                }
            }
        } else {
            // We want to tick only when tick_once is true (Space key sets this)
            if self.single_tick {
                // Tick the cpu
                match self.cpu.tick() {
                    Ok(n) => {
                        tick_happened = true;
                        self.last_pc = n;
                    }
                    Err(e) => {
                        panic!("Unable to tick {}", e);
                    }
                }
                self.single_tick = false;
            }
        }

        // If needed/wanted, call off to the disassembler to print some pretty details
        if self.cpu.disassemble && tick_happened {
            if self.cpu.cycle_count % 25 == 0 {
                println!("{}", disassembler::HEADER);
            }
            // Get our disassembler message text as well as our "next" opcode description
            let dt = disassembler::disassemble(&self.cpu, self.last_pc);
            println!("{}", dt);
            self.last_msg = dt;
        }
        Ok(())
    }
}

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

    let app = Arc::new(Mutex::new(Emu::new(rom_file)?));
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

        app_clone.lock().unwrap().create_display_texts(&mut canvas);

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

// Does what it says on the tin.
fn add_display_text(canvas: &mut sdl2::render::WindowCanvas, to_display: &str, x: i32, y: i32) {
    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font = ttf_context
        .load_font("./resources/fonts/OpenSans-Regular.ttf", 16)
        .unwrap();

    let surface = font.render(to_display).solid(BLACK).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

    canvas
        .copy(
            &texture,
            None,
            Some(Rect::new(x, y, surface.width(), surface.height())),
        )
        .unwrap();
}
