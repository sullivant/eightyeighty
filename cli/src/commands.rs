use std::cell::{Ref, RefCell};
use std::io;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use emulator::{Emulator, RunStopReason};
use emulator::devices::hardware::midway::{MidwayHardware, MidwayInput};

use crate::Keyboard;

pub static COMMANDS: &[Command] = &[
    Command {
        name: "quit",
        usage: "quit",
        help: "Exit the emulator",
        handler: cmd_quit,
    },
    Command {
        name: "exit",
        usage: "exit",
        help: "Exit the emulator",
        handler: cmd_quit,
    },
    Command {
        name: "step",
        usage: "step",
        help: "Single step the next instruction",
        handler: cmd_step,
    },
    Command {
        name: "run",
        usage: "run | run n",
        help: "Run forever, or n commands, or until HALT",
        handler: cmd_run,
    },
    Command {
        name: "hw",
        usage: "hw",
        help: "Show hardware (i/o latches) state",
        handler: cmd_hw,
    },
    Command {
        name: "regs",
        usage: "regs",
        help: "Show CPU registers",
        handler: cmd_regs,
    },
];

        // ["regs"] => regs(&emu.cpu),
        // ["emu"] => emu_state(emu),

pub struct Command {
    pub name: &'static str,
    pub usage: &'static str,
    pub help: &'static str,
    pub handler: fn(
        &mut Emulator,
        &Rc<RefCell<MidwayHardware>>,
        &[&str],
    ) -> bool, // return false => exit REPL
}

// This will send the input from rustyline off to the proper command handler
pub fn dispatch(emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, line: &str) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return true;  // No actual command.
    }

    let (name, args) = parts.split_first().unwrap();

    if let Some(cmd) = COMMANDS.iter().find(|c| c.name == *name) {
        (cmd.handler)(emu, hw, args) // Passed off to the handler
    } else {
        println!("Unknown command: {}", name);
        true
    }
}

fn cmd_quit(_emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str],) -> bool {
    false
}
 
fn cmd_step(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> bool {
    match emu.step() {
        Some(result) => {
            println!(
                "{:04X}: {:02X}  {:<10}  +{} cycles",
                result.pc,
                result.opcode,
                result.mnemonic,
                result.cycles
            );
        }
        _ => (),
    }
    true
}

fn cmd_run(
    emu: &mut Emulator,
    hw: &Rc<RefCell<MidwayHardware>>,
    args: &[&str],
) -> bool {
    match args {
        [] => {
            println!("Running forever. ESC to stop.");
            if let Err(e) = run_forever(emu, hw) {
                println!("Run error: {}", e);
            }
        },

        [cycles] => {
            let cycles: u64 = match cycles.parse() {
                Ok(c) => c,
                Err(_) => {
                    println!("Invalid cycle count: {}", cycles);
                    return true;
                }
            };

            match emu.run_blocking(Some(cycles)) {
                RunStopReason::CycleBudgetExhausted => { println!("Stopped: Cycle budget exhausted.");},
                RunStopReason::Halted => { println!("Stopped: Halted.");},
                _ => { println!("Stopped: Unknown reason.");}
            }
        },

        _ => {
            println!("Usage: run [cycles]");
        }
    }

    true
}

fn cmd_hw(_emu: &mut Emulator, hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str]) -> bool {
    show_hardware_state(hw.borrow());
    true
}

/// Displays registers
fn cmd_regs(emu: &mut Emulator, _hw: &Rc<RefCell<MidwayHardware>>, _args: &[&str],) -> bool {
    let cpu = &emu.cpu;
    println!(
        "A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X}",
        cpu.a, cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, cpu.sp, cpu.pc
    );

    true
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
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    keyboard.press(key.code); // Register the press

                    // And send the input off to the hardware
                    if let Some(input) = key_to_midway_input(key.code) {
                        hardware.borrow_mut().press(input);
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
        }

        // Now handle the releases; tick() returns a list of newly released
        // keys, we send them to the hardware in the form of releases.
        for key in keyboard.tick() {
            if let Some(input) = key_to_midway_input(key) {
                hardware.borrow_mut().release(input);
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

// This is used a lot when parsing command arguments
fn parse_u16_hex(s: &str) -> Option<u16> {
    u16::from_str_radix(s, 16).ok()
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
