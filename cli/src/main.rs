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
                // if !handle_command(&mut emu, &hardware, line) {
                if !dispatch(&mut emu, &hardware, line) {
                     break;
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

/// Just loads provided filepath into a vec.
fn load_rom_file(path: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
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
        ["int", line] => {
            if let Ok(r) = line.parse::<u8>() {
                emu.bus.request_interrupt(r);
            }
        },



        // Will resend the line, to be properly parsed in the mem fn.
        ["mem", _, _] => mem(&emu.bus, line),

        ["rom"] => print_rom(emu),

        ["pc"] => println!("PC = {:04X}", emu.cpu.pc),

        ["insert", rom_name] => {
            let file = if rom_name.ends_with(".rom") {
                rom_name.to_string()
            } else {
                format!("{}.rom", rom_name)
            };

            let path = format!("resources/roms/{}", file);
            println!("Inserting ROM and resetting: {}", path);

            // If it loads from the file, stuff it into the Emulator
            match load_rom_file(&path) {
                Ok(bytes) => {
                    emu.insert_rom(bytes);
                }
                Err(e) => {
                    println!("File error: {}", e);
                }
            }

            if let Err(e) = emu.reset() {
                println!("Error in resetting: {}",e);
                return false;
            }
        },

        ["remove"] => {
            println!("Removing ROM from Emulator");
            emu.remove_rom();
        },

        ["reset"] => {
            println!("Resetting Emulator");
            if let Err(e) = emu.reset() {
                println!("Error in resetting: {}",e);
                return false;
            }
        },

        ["break", addr] => {
            if let Ok(a) = u16::from_str_radix(addr, 16) {
                emu.add_breakpoint(a);
                println!("Breakpoint added at {:04X}", a);
            } else {
                println!("Invalid address");
            }
        },

        ["break", "remove", addr] => {
            if let Ok(a) = u16::from_str_radix(addr, 16) {
                emu.remove_breakpoint(a);
                println!("Breakpoint removed at {:04X}", a);
            } else {
                println!("Invalid address");
            }
        },

        ["breaklist"] => {
            let bps = emu.breakpoints();
            if bps.is_empty() {
                println!("No Breakpoints set.");
            } else {
                println!("Breakpoints:");
                for pc in bps {
                    println!("   {:04X}", pc);
                }
            }
        },

        ["set_port", port_str, value_str] => {
            match (port_str.parse::<u8>(), value_str.parse::<u8>()) {
                (Ok(port), Ok(value)) => {
                    hardware.borrow_mut().set_port(port, value);
                    println!("Set port {} to {:#04X}", port, value);
                }
                _ => println!("Usage: set_port <port: u8> <value: u8>"),
            }
        },

        ["set_bit", port_str, bit_str] => {
            match (port_str.parse::<u8>(), bit_str.parse::<u8>()) {
                (Ok(port), Ok(bit)) if bit < 8 => {
                    hardware.borrow_mut().set_bit(port, bit);
                    println!("Set bit {} on port {}", bit, port);
                }
                _ => println!("Usage: set_bit <port: u8> <bit: 0-7>"),
            }
        },

        ["clear_bit", port_str, bit_str] => {
            match (port_str.parse::<u8>(), bit_str.parse::<u8>()) {
                (Ok(port), Ok(bit)) if bit < 8 => {
                    hardware.borrow_mut().clear_bit(port, bit);
                    println!("Cleared bit {} on port {}", bit, port);
                }
                _ => println!("Usage: clear_bit <port: u8> <bit: 0-7>"),
            }
        },

        _ => println!("Unknown command: {}", line),
    }

    true
}




/// Displays emulator state
fn emu_state(emu: &mut Emulator) {
    match emu.run_state() {
        RunState::Running => { println!("State: Running");},
        RunState::Stopped => { println!("State: Stopped");},
    };
    match emu.cpu.interrupts_enabled() {
        true => { println!("Interrupts Enabled")},
        false=> { println!("Interrupts Not Enabled")},
    };
    match emu.bus.peek_interrupt() {
        Some(i) => { println!("Pending Interrupt: {}", i)},
        None => { println!("Pending Interrupt: None")},
    };
}

/// Displays a portion of memory
fn mem(bus: &Bus, cmd: &str) {
    let parts: Vec<_> = cmd.split_whitespace().collect();
    if parts.len() != 3 {
        println!("Usage: mem <addr> <len>");
        return;
    }

    let addr = usize::from_str_radix(parts[1], 16).unwrap_or(0);
    let len = parts[2].parse::<usize>().unwrap_or(0);
    let bytes_per_line = 16;

    for line_start in (0..len).step_by(bytes_per_line) {
        // Print the address at start of line
        print!("{:04X}:  ", addr + line_start);

        // Print hex bytes for this line
        for i in 0..bytes_per_line {
            let idx = line_start + i;
            if idx < len {
                let byte = bus.read(addr + idx);
                print!("{:02X} ", byte);
            } else {
                // Padding for incomplete line
                print!("   ");
            }
        }

        // Print ASCII characters for this line
        print!(" ");

        for i in 0..bytes_per_line {
            let idx = line_start + i;
            if idx < len {
                let byte = bus.read(addr + idx);
                // Show printable ASCII or '.' for non-printable
                let ch = if byte.is_ascii_graphic() || byte == b' ' {
                    byte as char
                } else {
                    '.'
                };
                print!("{}", ch);
            } else {
                // Padding for incomplete line
                print!(" ");
            }
        }

        println!();
    }
}

/// Prints the currently loaded ROM in case one is curious
fn print_rom(emu: &Emulator) {
    match emu.rom() {
        Some(rom) => {
            for (addr, byte) in rom.iter().enumerate() {
                println!("{:04X}: {:02X}", addr, byte);
            }
        }
        None => {
            println!("No ROM loaded.");
        }
    }
}



