mod cpu;
mod disassembler;
extern crate queues;

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam, Text};
use ggez::*;
use glam::Vec2;
use nalgebra as na;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::env;
use std::path;
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
pub const DISP_SCALE: f32 = 10.0;
pub const DISP_WIDTH: f32 = 640.0;
pub const DISP_HEIGHT: f32 = 480.0;
pub const DISP_HEIGHT_INFO_AREA: f32 = 200.0; // The added bottom info area for text
pub const MAX_MESSAGES_COUNT: usize = 4; // Max number of messages to add into the info area

#[derive(StructOpt)]
struct Cli {
    /// The input rom to look for
    rom: String,
}

pub struct App {
    dt: std::time::Duration,
    cpu: Cpu,
    cell: graphics::Mesh,
    texts: BTreeMap<&'static str, Text>,
    last_msg: String,
}

impl App {
    fn update_text_area(&mut self) {
        self.texts
            .insert("msg", Text::new(format!("last: {}", self.last_msg)));
    }

    fn new(ctx: &mut Context) -> GameResult<App> {
        let dt = std::time::Duration::new(0, 0);
        let black = graphics::Color::new(0.0, 0.0, 0.0, 1.0);
        let mut texts = BTreeMap::new(); // Setup some texts for update later

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
            // Store the text in `App`s map, for drawing in main loop.
            texts.insert("1_romname", Text::new(format!("ROM Loaded:{}", f)));

            println!(
                "Loaded rom file: {} start at: {:#06X} end at: {:#06X}",
                f,
                dims.0,
                dims.1 - 1
            );
        }

        // Setup a "cell"/pixel for the engine to use
        let cell = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, DISP_SCALE, DISP_SCALE),
            black,
        )?;

        // Return a good version of the app object
        Ok(App {
            dt,
            cpu,
            cell,
            texts,
            last_msg: "N/A".to_string(),
        })
    }
}

impl ggez::event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = timer::delta(ctx); // Frame count timer
        while timer::check_update_time(ctx, 80) {
            // Tick the cpu
            match self.cpu.tick() {
                Ok(_) => {}
                Err(e) => {
                    panic!("Unable to tick: {}", e);
                }
            }
            // If needed/wanted, call off to the disassembler to print some pretty details
            if self.cpu.disassemble {
                if self.cpu.cycle_count % 25 == 0 {
                    disassembler::print_header();
                }
                // Get our disassembler message text
                let dt = disassembler::disassemble(&self.cpu);
                println!("{}", dt);
                self.last_msg = dt;
            }
            self.update_text_area();
        }

        // Let our family know we are ok
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        let black = graphics::Color::new(0.0, 0.0, 0.0, 1.0);

        // Draw text objects/details
        // Create a little FPS text and display it in the info area
        let mut height = DISP_HEIGHT; // Start at the top of the info area

        // Draw a border line above info area
        let mut line = graphics::Mesh::new_line(
            ctx,
            &[na::Point2::new(0.0, 0.0), na::Point2::new(DISP_WIDTH, 0.0)],
            2.0,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &line, ([0.0, height],))?;
        line = graphics::Mesh::new_line(
            ctx,
            &[na::Point2::new(0.0, 0.0), na::Point2::new(0.0, height)],
            2.0,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &line, ([DISP_WIDTH, 0.0],))?;

        // A FPS timer (not a mapped obj because it changes rapidly)
        height += 2.0;
        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {}", fps));
        graphics::draw(ctx, &fps_display, (Vec2::new(0.0, height), black))?;

        // Draw the mapped text objects, too
        height += 2.0 + fps_display.height(ctx) as f32; // Prep height to be used for mapped objs
        for text in self.texts.values() {
            graphics::queue_text(ctx, text, Vec2::new(0.0, height), Some(black));
            height += 2.0 + text.height(ctx) as f32;
        }
        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn go() -> GameResult {
    // Create a window.
    let mut main_window = ContextBuilder::new("eightyeighty", "eightyeighty");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let path = path::PathBuf::from(manifest_dir).join("resources");
        println!("Adding 'resources' path {:?}", path);
        main_window = main_window.add_resource_path(path);
    }

    // Build our context
    let (mut ctx, mut event_loop) = main_window.build().unwrap();

    // Build our application
    let mut app = App::new(&mut ctx)?;

    // Run the application
    event::run(&mut ctx, &mut event_loop, &mut app)
}
