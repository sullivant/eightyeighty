use std::cell::Ref;
use std::cell::RefCell;
use std::io;
use std::fs;
use std::rc::Rc;

// For key handling
use std::collections::HashMap;
use crossterm::event::KeyEventKind;
use crossterm::event::{self, Event};
use crossterm::event::{KeyCode};
use std::time::{Duration, Instant};

// For REPL work
use rustyline::{error::ReadlineError};
use rustyline::DefaultEditor;

mod commands;
use commands::dispatch;

use emulator::bus::IoDevice;
use emulator::{RunState, RunStopReason};
use emulator::devices::hardware::midway::{MidwayHardware, MidwayInput};
use emulator::{self, Emulator, cpu::CPU, bus::Bus};

// A simple test rom with a few instructions to load at the start
const ROM_TST: &[u8] = &[0x3E, 0x42, 0x76];


struct HardwareProxy {
    hardware: Rc<RefCell<MidwayHardware>>,
}
impl IoDevice for HardwareProxy {
    fn input(&mut self, port: u8) -> u8 {
        self.hardware.borrow_mut().input(port)
    }
    fn output(&mut self, port: u8, value: u8) {
        self.hardware.borrow_mut().output(port, value)
    }

    fn set_port(&mut self, port: u8, value: u8) {
        self.hardware.borrow_mut().set_port(port, value);
    }

    fn set_bit(&mut self, port: u8, bit: u8) {
        self.hardware.borrow_mut().set_bit(port, bit);
    }
    fn clear_bit(&mut self, port: u8, bit: u8) {
        self.hardware.borrow_mut().clear_bit(port, bit);
    }
}

struct Keyboard {
    pressed: HashMap<KeyCode, Instant>,
    release_after: Duration,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            pressed: HashMap::new(),
            release_after: Duration::from_millis(50),
        }
    }

    /// Register a press or refresh if already pressed
    fn press(&mut self, key: KeyCode) {
        self.pressed.insert(key, Instant::now());
    }

    /// Returns keys that should be released by now 
    fn tick(&mut self) -> Vec<KeyCode> {
        let now = Instant::now();
        let mut released = Vec::new();

        // For each key see if it should be released by now by looking at
        // each key and its timestamp, if it should be released, 
        // do not retain it.  Simple.
        self.pressed.retain(|&key, &mut t| {
            if now.duration_since(t) > self.release_after {
                released.push(key);
                false
            } else {
                true
            }
        });

        released
    }
}





fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let prompt = "8080> ";

    // Store REPL history
    let history_path = ".history";
    let _ = rl.load_history(history_path);

    // Our "hardware" here:
    let hardware = Rc::new(RefCell::new(MidwayHardware::new()));
    println!("Original hardware Rc points to: {:p}", Rc::as_ptr(&hardware));

    // Which is used when setting up the emu.
    let mut emu: Emulator = setup_emu(&hardware)?;


    println!("Starting REPL...");
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line)?;

                // Handling of command also needs to know about the hardware because it's going to
                // read keys and set the proper ports.
                if !dispatch(&mut emu, &hardware, line) {
                    break; // If we have returned false from dispatch, we were instructed to quit the REPL
                }
            }

            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }

            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(history_path);
    Ok(())
}



/// Will create the emulator machine, and insert the "default" ROM
fn setup_emu(hardware: &Rc<RefCell<MidwayHardware>>) -> Result<Emulator, String> {
    println!("Creating emulator...");
    
    // let hw_proxy = HardwareProxy { hardware: hardware.clone() };
    // println!("HardwareProxy pointer before Box: {:p}", &*hw_proxy.hardware);
    // let boxed_io: Box<dyn IoDevice> = Box::new(hw_proxy);

    // println!("Box<dyn IoDevice> pointer before moving to Emulator:");
    // let raw_ptr = &*boxed_io as *const dyn IoDevice;
    // let (data_ptr, _vtable): (*const (), *const ()) = unsafe { std::mem::transmute(raw_ptr) };
    // println!("data_ptr: {:p}", data_ptr);

    // Box up the hardware proxy, with a cloned version of the hardware, and create an emu with it.
    let mut emu = Emulator::with_io(Box::new(HardwareProxy { hardware: hardware.clone(),}));
    // let mut emu = Emulator::with_io(boxed_io);

    println!("Inserting ROM and loading...");
    emu.load_rom(ROM_TST.to_vec())?;

    Ok(emu)
}

/// Actually handles processing the REPL command
fn handle_command(emu: &mut Emulator, hardware: &Rc<RefCell<MidwayHardware>>, line: &str) -> bool {

    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts.as_slice() {


        _ => println!("Unknown command: {}", line),
    }

    true
}




