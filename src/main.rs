use egui::Checkbox;
use egui_sdl2_gl as egui_backend;
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};

use std::time::Instant;
use sdl2::video::SwapInterval;

fn main() -> Result<(), String> {
    lib::go()?;
    Ok(())
}
