mod cpu;
mod disassembler;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::collections::BTreeMap;
use std::env;
use std::path;
use std::time::Duration;
use structopt::StructOpt;

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
pub const DISP_WIDTH: f32 = 640.0; // Overall width/height
pub const DISP_HEIGHT: f32 = 480.0;
pub const EMU_WIDTH: u16 = 224; // Emulator display area width/height
pub const EMU_HEIGHT: u16 = 256;
pub const CELL_SIZE: u32 = 2; // The size of a "cell" or pixel
const WHITE: Color = Color::RGB(255, 255, 255);
const BLACK: Color = Color::RGB(0, 0, 0);
const RED: Color = Color::RGB(255, 0, 0);
const GREEN: Color = Color::RGB(0, 255, 0);

#[derive(StructOpt)]
struct Cli {
    //TODO: Implement this for a CLI option
    /// The input rom to look for
    rom: String,
}

pub struct App {
    dt: std::time::Duration,
    cpu: Cpu,
    last_msg: String, // Contains last disassembler message
    next_msg: String, // Contains next disassembler command to be run
    last_pc: usize,
    pause_on_tick: bool,
    single_tick: bool,
}

impl App {
    fn new() -> Result<App, String> {
        println!("Creating new App Object");
        let dt = std::time::Duration::new(0, 0);

        // Generate our CPU
        let mut cpu = Cpu::new();
        cpu.set_disassemble(true);
        cpu.set_nop(true);

        // The list of rom files to load for this particular collection/game
        let rom_files: [String; 1] = [
            //String::from("./resources/roms/TST8080.COM"),
            String::from("./resources/roms/INVADERS.COM"),
        ];
        let mut dims: (usize, usize) = (0, 0);

        for f in rom_files {
            match cpu.load_rom(f.clone(), dims.1) {
                Ok(i) => {
                    dims = i;
                }
                Err(err) => {
                    panic!("Unable to load rom file {}: {}", f, err);
                }
            }

            println!(
                "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
                f,
                dims.0,
                dims.1 - 1
            );
        }

        // Return a good version of the app object
        Ok(App {
            dt,
            cpu,
            last_msg: "N/A".to_string(),
            next_msg: "N/A".to_string(),
            last_pc: 0,
            pause_on_tick: true,
            single_tick: false,
        })
    }

    fn update(&mut self) {
        let mut tick_happened: bool = false;
        match self.cpu.tick() {
            Ok(n) => {
                tick_happened = true;
                self.last_pc = n;
            }
            Err(e) => {
                panic!("Unable to tick: {}", e);
            }
        }

        // If needed/wanted, call off to the disassembler to print some pretty details
        if self.cpu.disassemble && tick_happened {
            if self.cpu.cycle_count % 25 == 0 {
                disassembler::print_header();
            }
            // Get our disassembler message text as well as our "next" opcode description
            let dt = disassembler::disassemble(&self.cpu, self.last_pc);
            let ndt = disassembler::get_opcode_text(self.cpu.next_opcode);
            println!("{}", dt);
            self.last_msg = dt;
            self.next_msg = format!(
                "{} (op:{:#04X}/{:08b},dl:{:#04X},dh:{:#04X})",
                ndt,
                self.cpu.next_opcode.0,
                self.cpu.next_opcode.0,
                self.cpu.next_opcode.1,
                self.cpu.next_opcode.2
            ); // We only really care about the text
        }
    }

    // fn update(&mut self) {
    //     while timer::check_update_time(ctx, 40) {
    //         let mut tick_happened: bool = false;
    //         // If we are not in pause_on_tick mode, tick away
    //         if !self.pause_on_tick {
    //             // Tick the cpu
    //             match self.cpu.tick() {
    //                 Ok(n) => {
    //                     tick_happened = true;
    //                     self.last_pc = n;
    //                 }
    //                 Err(e) => {
    //                     panic!("Unable to tick: {}", e);
    //                 }
    //             }
    //         } else {
    //             // We want to tick only when tick_once is true (Space key sets this)
    //             if self.single_tick {
    //                 // Tick the cpu
    //                 match self.cpu.tick() {
    //                     Ok(n) => {
    //                         tick_happened = true;
    //                         self.last_pc = n;
    //                     }
    //                     Err(e) => {
    //                         panic!("Unable to tick: {}", e);
    //                     }
    //                 }
    //                 self.single_tick = false;
    //             }
    //         }

    //         // If needed/wanted, call off to the disassembler to print some pretty details
    //         if self.cpu.disassemble && tick_happened {
    //             if self.cpu.cycle_count % 25 == 0 {
    //                 disassembler::print_header();
    //             }
    //             // Get our disassembler message text as well as our "next" opcode description
    //             let dt = disassembler::disassemble(&self.cpu, self.last_pc);
    //             let ndt = disassembler::get_opcode_text(self.cpu.next_opcode);
    //             println!("{}", dt);
    //             self.last_msg = dt;
    //             self.next_msg = format!(
    //                 "{} (op:{:#04X}/{:08b},dl:{:#04X},dh:{:#04X})",
    //                 ndt,
    //                 self.cpu.next_opcode.0,
    //                 self.cpu.next_opcode.0,
    //                 self.cpu.next_opcode.1,
    //                 self.cpu.next_opcode.2
    //             ); // We only really care about the text
    //         }
    //     }
    // }
}

pub fn go() -> Result<(), String> {
    // Create a window.
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(
            "8080",
            DISP_WIDTH as u32 * CELL_SIZE,
            DISP_HEIGHT as u32 * CELL_SIZE,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    canvas.clear();
    canvas.present();
    // Build our application
    let mut app = App::new()?;

    // Main loop
    'running: loop {
        // Hit up the event pump
        for event in event_pump.poll_iter() {
            // Read the keyboard
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Tick the CPU
        // TODO - Might want this in its own thread, like as seen elsewhere and started earlier in
        // the go() fn before this loop
        app.update();

        // Clear the screen
        canvas.clear();

        // Draw the graphics portion (TODO)
        canvas.set_draw_color(BLACK);
        canvas.draw_line(Point::new(100, 100), Point::new(200, 200))?;
        // canvas
        //     .fill_rect(Rect::new(100, 100, 2 as u32, 2 as u32))
        //     .unwrap();

        // Present the updated screen
        canvas.set_draw_color(WHITE);
        canvas.present();

        // Sleep a bit
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!("Shutting down. Final CPU state:\n{}", app.cpu);
    Ok(())
}
