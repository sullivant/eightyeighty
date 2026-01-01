use std::cell::{Ref, RefCell};
use std::{fs, io};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use emulator::bus::IoDevice;
use emulator::{Emulator, RunState, RunStopReason};
use emulator::devices::hardware::midway::{MidwayHardware, MidwayInput};

use crate::Keyboard;

// To clean things we make a type alias
pub type CommandHandler = fn(
    &mut Emulator,
    &Rc<RefCell<MidwayHardware>>,
    &[&str],
) -> Result<bool, String>;

pub struct Command {
    // pub name: &'static str,
    pub names: &'static [&'static str],
    pub usage: &'static str,
    pub help: &'static str,
    pub handler: CommandHandler, 
}

pub static COMMANDS: &[Command] = &[
    Command {
        names: &["help", "?"],
        usage: "help [command]",
        help: "List available commands or details if [command] is provided.",
        handler: cmd_help,
    },
    Command {
        names: &["quit", "exit"],
        usage: "quit",
        help: "Exit the emulator",
        handler: cmd_quit,
    },
    Command {
        names: &["step"],
        usage: "step",
        help: "Single step the next instruction",
        handler: cmd_step,
    },
    Command {
        names: &["run"],
        usage: "run [n]",
        help: "Run the emulator. If n is provided, run for n instructions; otherwise run until HALT.",
        handler: cmd_run,
    },
    Command {
        names: &["hw"],
        usage: "hw",
        help: "Show hardware (i/o latches) state",
        handler: cmd_hw,
    },
    Command {
        names: &["regs"],
        usage: "regs",
        help: "Show CPU registers",
        handler: cmd_regs,
    },
    Command {
        names: &["emu"],
        usage: "emu",
        help: "Show Emulator State",
        handler: cmd_emu_state,
    },
    Command {
        names: &["mem"],
        usage: "mem <loc> <len>",
        help: "Show Memory starting at <loc> and for <len> bytes",
        handler: cmd_mem,
    },
    Command {
        names: &["int"],
        usage: "int <n>",
        help: "Request interrupt at port <n>",
        handler: cmd_int,
    },
    Command {
        names: &["rom"],
        usage: "rom",
        help: "Show contents of loaded ROM",
        handler: cmd_rom,
    },
    Command {
        names: &["pc"],
        usage: "pc",
        help: "Show the current Program Counter (PC) value",
        handler: cmd_pc,
    },
    Command {
        names: &["insert"],
        usage: "insert <ROM filename>",
        help: "Inserts ROM into emulator and issues a reset",
        handler: cmd_insert,
    },
    Command {
        names: &["remove"],
        usage: "remove",
        help: "Removes any inserted ROM from the emulator",
        handler: cmd_remove,
    },
    Command {
        names: &["reset"],
        usage: "reset",
        help: "Resets emulator to initial state and loads rom",
        handler: cmd_reset,
    },
    Command {
        names: &["break"],
        usage: "break <add <pc> | rm <pc> | ls>",
        help: "Adds or removes breakpoints at <pc> location, or lists existing ones",
        handler: cmd_break,
    },
    Command {
        names: &["setport", "sp"],
        usage: "setport <port> <value>",
        help: "Sets port <num> to <value>",
        handler: cmd_set_port,
    },
    Command {
        names: &["setbit", "sb"],
        usage: "setbit <num> <bit>",
        help: "Sets <bit> on port <num>",
        handler: cmd_set_bit,
    },
    Command {
        names: &["clearbit", "cb"],
        usage: "clearbit <num> <bit>",
        help: "Clears <bit> on port <num>",
        handler: cmd_clear_bit,
    },
];

/// This will send the input from rustyline off to the proper command handler.  If a command's
/// handler returns true, all is well.  However, if the handler returns false, dispatch here
/// will print the command's usage string.
pub fn dispatch(emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, line: &str) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let (name, args) = match parts.split_first() {
        Some(v) => v,
        None => return true,
    };

    let Some(cmd) = COMMANDS.iter().find(|c| c.names.contains(name)) else {
        println!("Unknown command: {}", name);
        return true;
    };

    // If this is Ok(false) we can bail and quit.
    let ok: Result<bool, String> = (cmd.handler)(emu, hw, args);

    match ok {
        Ok(true) => { 
            // Command was a success
            true 
        }, 
        Ok(false) => { 
            // Command indicated "quit REPL/emu"
            false
        },
        Err(s) => {
            // Command was in error
            println!("Error: {}", s);
            println!("Usage: {}", cmd.usage);
            true
        }
    }
}

// Allows for searching the static array for a provided command
fn find_command(name: &str) -> Option<&'static Command> {
    // COMMANDS.iter().find(|c| c.name == name)
    COMMANDS.iter().find(|c| c.names.contains(&name))
}

/// Walks the COMMANDS array and pretty prints the commands, descriptions, and usage examples.
fn cmd_help(_emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [] => {
            println!("Available commands:");
            for c in COMMANDS {
                println!("  {:<15} {}", c.names.join(" | "), c.help);
            }
        }
        [name] => {
            if let Some(c) = find_command(name) {
                println!("{}\n\nUsage:\n  {}", c.help, c.usage);
            } else {
                println!("Unknown command: {}", name);
            }
        }
        _ => println!("Usage: help [command]"),
    }

    Ok(true)
}


fn cmd_quit(_emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str],) -> Result<bool, String> {
    Ok(false)
}
 
fn cmd_step(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
    if let Some(result) = emu.step() {
       println!(
                "{:04X}: {:02X}  {:<10}  +{} cycles",
                result.pc,
                result.opcode,
                result.mnemonic,
                result.cycles
            );
    }

    Ok(true)
}

fn cmd_run(emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [] => {
            println!("Running forever. ESC to stop.");
            if let Err(e) = run_forever(emu, hw) {
                return Err(format!("Run error: {}", e));
            }
        },

        [cycles] => {
            let cycles: u64 = match cycles.parse() {
                Ok(c) => c,
                Err(_) => {
                    return Err(format!("Invalid cycle count: {}", cycles));
                }
            };

            match emu.run_blocking(Some(cycles)) {
                RunStopReason::CycleBudgetExhausted => { println!("Stopped: Cycle budget exhausted.");},
                RunStopReason::Halted => { println!("Stopped: Halted.");},
                _ => { println!("Stopped: Unknown reason.");}
            }
        },

        _ => {
            return Err("Invalid argument".to_string());
        }
    }

    Ok(true)
}

fn cmd_hw(_emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
    show_hardware_state(hw.borrow());
    Ok(true)
}

/// Displays registers
fn cmd_regs(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str],) -> Result<bool, String> {
    let cpu = &emu.cpu;
    println!(
        "A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X}",
        cpu.a, cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, cpu.sp, cpu.pc
    );

    Ok(true)
}

fn cmd_emu_state(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str],) -> Result<bool, String> {
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

    Ok(true)
}

fn cmd_mem(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    let (addr, len): (usize, usize) = match args {
        [addr, len] => {
            let addr = match usize::from_str_radix(addr, 16) {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!("Invalid address: {}", addr));
                }
            };

            let len = match len.parse::<usize>() {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!("Invalid length: {}", len));
                }
            };

            (addr, len)
        }

        _ => {
            return Err("Invalid arguments".to_string());
        }
    };

    let bytes_per_line = 16;

    for line_start in (0..len).step_by(bytes_per_line) {
        // Print the address at start of line
        print!("{:04X}:  ", addr + line_start);

        // Print hex bytes for this line
        for i in 0..bytes_per_line {
            let idx = line_start + i;
            if idx < len {
                let byte = emu.bus.read(addr + idx);
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
                let byte = emu.bus.read(addr + idx);
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

    Ok(true)
}

fn cmd_int(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    let addr: u8 = match args {
        [addr] => {
             match u8::from_str_radix(addr, 8) {
                Ok(r) => r,
                _ => { return Err("Invalid argument".to_string());}
             }
        },
        _ => {
            return Err("Invalid argument".to_string());
        }
    };

    emu.bus.request_interrupt(addr);

    Ok(true)
}

fn cmd_rom(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
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
    Ok(true)
}

fn cmd_pc(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
    println!("PC = {:04X}", emu.cpu.pc);
    Ok(true)
}

fn cmd_insert(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    let rom_name: String = match args {
        [addr] => {
             addr.to_string()
        },
        _ => {
            return Err("Invalid argument".to_string());
        }
    };

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
        return Err(format!("Error in resetting: {}",e));
    }

    Ok(true)
}

fn cmd_remove(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
    println!("Removing ROM from Emulator");
    emu.remove_rom();

    Ok(true)
}

fn cmd_reset(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> Result<bool, String> {
    println!("Resetting Emulator");
    if let Err(e) = emu.reset() {
        return Err(format!("Error in resetting: {}",e));
    }
    Ok(true)
}

fn cmd_break(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [] => {
            println!("Listing breakpoints.");
        },

        ["add", a] => {
            if let Ok(a) = u16::from_str_radix(a, 16) {
                emu.add_breakpoint(a);
                println!("Breakpoint added at {:04X}", a);
            } else {
                println!("Invalid address");
            }
        },

        ["rm", a] => {
            if let Ok(a) = u16::from_str_radix(a, 16) {
                emu.remove_breakpoint(a);
                println!("Breakpoint removed at {:04X}", a);
            } else {
                println!("Invalid address");
            }
        },

        ["ls"] => {
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

        _ => {
            println!("Usage: run [cycles]");
        }
    }

    Ok(true)
}

fn cmd_set_port(_emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [port, value] => {
            match (port.parse::<u8>(), value.parse::<u8>()) {
                (Ok(port), Ok(value)) => {
                    hw.borrow_mut().set_port(port, value);
                    println!("Set port {} to {:#04X}", port, value);
                }
                _ => return Err("Invalid arguments".to_string()),
            }
        },
        _ => {
            return Err("Invalid arguments".to_string());
        }
    };
    
    Ok(true)
}

fn cmd_set_bit(_emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [port, value] => {
            match (port.parse::<u8>(), value.parse::<u8>()) {
                (Ok(port), Ok(value)) => {
                    hw.borrow_mut().set_bit(port, value);
                    println!("Set port {} to {:#04X}", port, value);
                }
                _ => return Err("Invalid arguments".to_string()),
            }
        },
        _ => {
            return Err("Invalid arguments".to_string());
        }
    };
    
    Ok(true)
}

fn cmd_clear_bit(_emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, args: &[&str]) -> Result<bool, String> {
    match args {
        [port, value] => {
            match (port.parse::<u8>(), value.parse::<u8>()) {
                (Ok(port), Ok(value)) => {
                    hw.borrow_mut().clear_bit(port, value);
                    println!("Set port {} to {:#04X}", port, value);
                }
                _ => return Err("Invalid arguments".to_string()),
            }
        },
        _ => {
            return Err("Invalid arguments".to_string());
        }
    }
    
    Ok(true)
}

/// Runs forever, processing keyboard events while doing so.
fn run_forever(emu: &mut Emulator, hardware: &Rc<RefCell<MidwayHardware>>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let mut keyboard = Keyboard::new();
    let mut last_tick = Instant::now();

    loop {
        // Run a slice of deep dish
        let stop_reason = emu.run_blocking(Some(2_000));

        if let RunStopReason::Breakpoint(pc) = stop_reason {
            crossterm::terminal::disable_raw_mode()?;
            println!("*** BREAKPOINT HIT at PC = {:04X} ***", pc);
            break;
        }

        if let RunStopReason::Halted = stop_reason {
            crossterm::terminal::disable_raw_mode()?;
            println!("CPU Halted; Stopping execution.");
            break;
        }

        // Poll input (non-blocking)
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
                keyboard.press(key.code); // Register the press

                // And send the input off to the hardware
                if let Some(input) = key_to_midway_input(key.code) {
                    hardware.borrow_mut().press(&input);
                    debug_keypress(key.code, hardware.borrow(), true)?;
                }

                // Escape to quit
                if key.code == KeyCode::Esc {
                    crossterm::terminal::disable_raw_mode()?;
                    return Ok(());
                }

                // CTRL+h to show status of the current input latches
                if key.code == KeyCode::Char('h') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    crossterm::terminal::disable_raw_mode()?;
                    let hw = hardware.borrow();
                    show_hardware_state(hw);
                    crossterm::terminal::enable_raw_mode()?;
                }
            }
        }

        // Now handle the releases; tick() returns a list of newly released
        // keys, we send them to the hardware in the form of releases.
        for key in keyboard.tick() {
            if let Some(input) = key_to_midway_input(key) {
                hardware.borrow_mut().release(&input);
                debug_keypress(key, hardware.borrow(), false)?;
            }
        }

        // Simple timing throttle for now
        if last_tick.elapsed() < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(1));
        }
        last_tick = Instant::now();
    }

    Ok(())
}

/// Just loads provided filepath into a vec.
fn load_rom_file(path: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
}

/// Maps keyboard codes to Midway specific inputs when we are using midway hardware.
/// I will probably want this in some way stuffed into hardware::midway.rs but without
/// reliance on crossterm::KeyCode.
fn key_to_midway_input(key: KeyCode) -> Option<MidwayInput> {
   use MidwayInput::*;
   match key {
    KeyCode::Char('c')  => Some(Coin),
    KeyCode::Char('1')  => Some(Start1),
    KeyCode::Char('2')  => Some(Start2),
    KeyCode::Left       => Some(Left),
    KeyCode::Right      => Some(Right),
    KeyCode::Char(' ')  => Some(Fire),
    _                   => None,    
   } 
}

fn debug_keypress(key: KeyCode, hw: Ref<'_, MidwayHardware>, pressed: bool) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    println!("------------------");                  
    if pressed { println!("Pressed: {:?}", key); } else { println!("Released: {:?}", key)}
    let hw = hw;
    show_hardware_state(hw);  
    println!("------------------");                  
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

// Shows input latch hardware state
fn show_hardware_state(hw: Ref<'_, MidwayHardware>) {
    println!("Input Latch 0: {:08b}", hw.input_latch0.read());
    println!("Input Latch 1: {:08b}", hw.input_latch1.read());
    println!("Input Latch 2: {:08b}", hw.input_latch2.read());
}
