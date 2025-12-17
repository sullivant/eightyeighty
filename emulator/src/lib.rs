#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use std::path::Path;
use std::io;

use cpu::CPU;
use memory::Memory;

pub struct Emulator {
    pub cpu: CPU,
    pub memory: Memory,
}


// impl Emulator {
//     pub fn new() -> Self {
        
//     }
// }