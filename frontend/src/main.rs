slint::include_modules!();

use std::cell::RefCell;
use std::{fs, io};
use std::rc::Rc;
use std::time::Duration;
use rfd::FileDialog;

// Allows for integration with the running system as if it was MidwayHardware.
use emulator::bus::IoDevice;
use emulator::devices::hardware::midway::{MidwayHardware, MidwayInput};
use emulator::{self, Emulator, RunState, RunStopReason};
use slint::platform::Key;
use slint::{ToSharedString, VecModel};
use slint::{ModelRc, SharedString};
use slint::{SharedPixelBuffer, Rgba8Pixel, Image}; // For wiring in display for now.  Later we'll wire into hardware/midway.rs

const WINDOW_SIZE_BYTES: usize = 256;

const CYCLES_PER_FRAME: u64 = 33_333; // Will hopefully work out around 2MHZ
const HALF_CYCLES_PER_FRAME: u64 = 16_667; // For dealing with RST1 and RST2

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




fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let ui_weak_all = ui.as_weak();

    // Gather a connection to the MidwayHardware to be used in the EMU
    let hardware = Rc::new(RefCell::new(MidwayHardware::new()));
    println!("Original hardware Rc points to: {:p}", Rc::as_ptr(&hardware));

    // Which is used when setting up the emu.
    let emu = Rc::new(RefCell::new(setup_emu(&hardware)?));

    // Clones of the emu (RC) for each closure
    let emu_for_all = emu.clone();

    // For handling of the memory window
    let memory_window: Rc<RefCell<Option<MemoryView>>> = Rc::new(RefCell::new(None));
    let memory_weak: Rc<RefCell<Option<slint::Weak<MemoryView>>>> = Rc::new(RefCell::new(None));

    // A timer that allows periodic syncing
    let ui_weak_sync = ui.as_weak();
    let ui_sync_timer = slint::Timer::default();
    let emu_for_timer = emu.clone();
    let ui_weak_display = ui.as_weak();
    ui_sync_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),  // ~60 FPS
        move || {
            if let Some(ui) = ui_weak_sync.upgrade() {
                // If runstate is running, run a small chunk of work.  Borrow happens only within
                // this block, so later, invoke_sync() can borrow again on its own.
                {
                    let mut emu = emu_for_timer.borrow_mut();
                    if emu.run_state() == RunState::Running {

                        // Run the first half of the frame and fire RST1
                        let stop_reason = emu.run_blocking(Some(CYCLES_PER_FRAME));

                        match stop_reason {
                            RunStopReason::Breakpoint(pc) => {
                                println!("*** BREAKPOINT HIT at PC = {:04X} ***", pc);
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::Halted => {
                                println!("CPU Halted; Stopping execution.");
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::Error => {
                                println!("CPU Halted with ERROR; Stopping execution.");
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::CycleBudgetExhausted => {
                                // Cool beans.  We can just keep running.
                                emu.set_run_state(RunState::Running);
                            }
                        }     

                        // Fire interrupt RST1
                        emu.bus.request_interrupt(1);

                        // Run the second half and fire RST2 
                        let stop_reason = emu.run_blocking(Some(HALF_CYCLES_PER_FRAME));

                        match stop_reason {
                            RunStopReason::Breakpoint(pc) => {
                                println!("*** BREAKPOINT HIT at PC = {:04X} ***", pc);
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::Halted => {
                                println!("CPU Halted; Stopping execution.");
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::Error => {
                                println!("CPU Halted with ERROR; Stopping execution.");
                                emu.set_run_state(RunState::Stopped);
                            },
                            RunStopReason::CycleBudgetExhausted => {
                                // Cool beans.  We can just keep running.
                                emu.set_run_state(RunState::Running);
                            }
                        }    

                        // Fire interrupt RST2
                        emu.bus.request_interrupt(2);
                                      
                    }
                }

                // Sync the UI
                ui.global::<AppLogic>().invoke_sync();
            }
        },
    );


    // A memory specific timer because it has a heavier lift.
    let window_start_addr = Rc::new(RefCell::new(0usize));
    let memory_timer = slint::Timer::default();
    let emu_for_mem_timer = emu.clone();
    let window_start_for_mem = window_start_addr.clone();
    let memory_weak_timer = memory_weak.clone();
    let emu_for_display = emu.clone();
    memory_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1), // 1 FPS
        move || {

            //TODO: May want to care here about if the memory view window is displayed - no need to update if it is not.

            let start = *window_start_for_mem.borrow();

            if let Some(weak) = memory_weak_timer.borrow().as_ref() {
                update_memory_view(weak, &emu_for_mem_timer, start, WINDOW_SIZE_BYTES);
            }
        }
    );

    // Handle syncing everything, except memory view portion, at once.
    let hw_for_sync = hardware.clone();
    ui.global::<AppLogic>().on_sync(move || {
        let Some(ui) = ui_weak_all.upgrade() else {
            return;
        };

        {
            let mut emu = emu_for_all.borrow_mut();
            let cpu = &emu.cpu;

            // Update registers
            let regs = ui.global::<EmulatorRegisters>();
            regs.set_a(cpu.a as i32);
            regs.set_b(cpu.b as i32);
            regs.set_c(cpu.c as i32);
            regs.set_e(cpu.e as i32);
            regs.set_h(cpu.h as i32);
            regs.set_l(cpu.l as i32);
            regs.set_sp(cpu.sp as i32);
            regs.set_sp_hex(format!("{:X}", cpu.sp).to_shared_string());
            regs.set_pc(cpu.pc as i32);
            regs.set_pc_hex(format!("{:X}", cpu.pc).to_shared_string());

            // Update emulator state
            let state = ui.global::<EmulatorState>();
            match emu.run_state() {
                RunState::Running => { state.set_state("State: Running".to_shared_string());},
                RunState::Stopped => { state.set_state("State: Stopped".to_shared_string());},
            };
            match emu.cpu.interrupts_enabled() {
                true => { state.set_interrupts("Interrupts Enabled".to_shared_string())},
                false=> { state.set_interrupts("Interrupts Not Enabled".to_shared_string())},
            };
            match emu.bus.peek_interrupt() {
                Some(i) => { state.set_pending(format!("Pending Interrupt: {}", i).to_shared_string())},
                None => { state.set_pending("Pending Interrupt: None".to_shared_string())},
            };
        }

        // Update the hardware state portion of the UI.
        {
            let state = ui.global::<HardwareState>();
            let hw = hw_for_sync.borrow();
            state.set_latch_0(format!("Input 0: {:08b}", hw.input_latch0.read()).to_shared_string());
            state.set_latch_1(format!("Input 1: {:08b}", hw.input_latch1.read()).to_shared_string());
            state.set_latch_2(format!("Input 2: {:08b}", hw.input_latch2.read()).to_shared_string());
        }

        // Update the display portion of the UI.
        {
            if let Some(ui) = ui_weak_display.upgrade() {
                let emu = emu_for_display.borrow();

                let frame = build_frame_from_vram(&emu);
                ui.set_frame(frame);
            }
        }
    });

    let window_for_actions = window_start_addr.clone();

    // Specific button / action handlers.
    let emu_for_step = emu.clone();
    let window_step = window_for_actions.clone();
    let memory_weak_step = memory_weak.clone();
    ui.global::<AppLogic>().on_cb_step(move || {
        {
            let mut emu = emu_for_step.borrow_mut();
            emu.step();  
        }  
        // Refresh memory view
        let start = *window_step.borrow();
        if let Some(weak) = memory_weak_step.borrow().as_ref() {
            update_memory_view(weak, &emu_for_step, start, WINDOW_SIZE_BYTES);
        }
    });

    let emu_for_reset = emu.clone();
    let window_reset: Rc<RefCell<usize>> = window_for_actions.clone();
    let memory_weak_reset = memory_weak.clone();
    ui.global::<AppLogic>().on_cb_reset(move || {
        {
            let mut emu = emu_for_reset.borrow_mut();
            emu.reset().unwrap();
        }
        let start = *window_reset.borrow();
        if let Some(weak) = memory_weak_reset.borrow().as_ref() {
            update_memory_view(weak, &emu_for_reset, start, WINDOW_SIZE_BYTES);
        }
    });

    let emu_for_run = emu.clone();
    ui.global::<AppLogic>().on_cb_run(move || {
        let mut emu = emu_for_run.borrow_mut();
        emu.set_run_state(RunState::Running);
    });

    let emu_for_stop = emu.clone();
    let window_stop = window_for_actions.clone();
    let memory_weak_stop = memory_weak.clone();
    ui.global::<AppLogic>().on_cb_stop(move || {
        {
            let mut emu = emu_for_stop.borrow_mut();
            emu.set_run_state(RunState::Stopped);
        }

        let start = *window_stop.borrow();
        if let Some(weak) = memory_weak_stop.borrow().as_ref() {
            update_memory_view(weak, &emu_for_stop, start, WINDOW_SIZE_BYTES);
        }
    });

    // Handle the request to cleanly exit the app or show settings
    ui.global::<AppLogic>().on_cb_exit(|| slint::quit_event_loop().unwrap() );
    let settings_window: Rc<std::cell::RefCell<Option<SettingsWindow>>> = 
        Rc::new(std::cell::RefCell::new(None));
    let settings_clone = settings_window.clone(); // We do stuff with the clone.
    ui.global::<AppLogic>().on_cb_show_settings(move || {
        let mut slot = settings_clone.borrow_mut();

        if slot.is_none() {
            let win = SettingsWindow::new().unwrap();
            win.show().unwrap();
            *slot = Some(win); // This lets us be reentrant.
        } else {
            slot.as_ref().unwrap().show().unwrap();
        }
    });

    // This handle deals with inserting a ROM via the CPU menu.
    let emu_for_rom = emu.clone();
    ui.global::<AppLogic>().on_cb_insert_rom(move || {
        if let Some(path) = FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .set_title("Select ROM to insert")
            .pick_file()
        {
            println!("Selected file: {:?}", path);

            match load_rom_file(&path.to_str().unwrap()) {
                Ok(bytes) => {
                    let mut emu = emu_for_rom.borrow_mut();
                    emu.insert_rom(bytes);
                    emu.reset().unwrap();
                }
                Err(e) => {
                    println!("File error: {}", e);
                }
            }
        }

    });


    // Handle keyboard input;
    // I do not like how this is not KeyEvent and instead is just the text representation.
    let hw_for_press = hardware.clone();
    let hw_for_release = hardware.clone();
    {
        ui.on_key_pressed(move |keytext: SharedString| {
            // println!("Key pressed: {:?}", keytext);
            if let Some(input) = slint_key_to_midway_input(&keytext) {
                handle_input(&hw_for_press, input, true);
            }        
        });
        ui.on_key_released(move |keytext: SharedString| {
            // println!("Key released: {:?}", keytext);
            if let Some(input) = slint_key_to_midway_input(&keytext) {
                handle_input(&hw_for_release, input, false);
            } 
        });
    }

    // This handle deals with popping the memory view when desired.
    let emu_for_memory = emu.clone();
    let window_start_addr_for_memory = window_start_addr.clone();
    let memory_window_clone = memory_window.clone();
    let memory_weak_clone = memory_weak.clone();
    ui.global::<AppLogic>().on_cb_show_memory(move || {
        // let win_ref: &MemoryView;

        {
            let mut slot = memory_window_clone.borrow_mut();

            if slot.is_none() {
                let new_win = MemoryView::new().unwrap();

                // Callback handling for the memory window PREVIOUS button
                {
                    let emu_prev = emu_for_memory.clone();
                    let window_prev = window_start_addr_for_memory.clone();
                    let memory_weak_prev = memory_weak_clone.clone();

                    new_win.global::<AppLogic>().on_previous_page(move || {
                        let mut start = window_prev.borrow_mut();
                        if *start >= WINDOW_SIZE_BYTES {
                            *start -= WINDOW_SIZE_BYTES;
                        } else {
                            *start = 0;
                        }
                        if let Some(weak) = memory_weak_prev.borrow().as_ref() {
                            update_memory_view(weak, &emu_prev, *start, WINDOW_SIZE_BYTES);
                        }
                    });
                }
                // Callback handling for the memory window NEXT button
                {
                    let emu_next = emu_for_memory.clone();
                    let window_next = window_start_addr_for_memory.clone();
                    let memory_weak_next = memory_weak_clone.clone();
                    let mem_len = emu_next.borrow().bus.memory().get_data().len();
                    new_win.global::<AppLogic>().on_next_page(move || {
                        let mut start = window_next.borrow_mut();

                        if *start + WINDOW_SIZE_BYTES < mem_len {
                            *start += WINDOW_SIZE_BYTES;
                        }

                        if *start + WINDOW_SIZE_BYTES > mem_len {
                            *start = mem_len.saturating_sub(WINDOW_SIZE_BYTES);
                        }
                        if let Some(weak) = memory_weak_next.borrow().as_ref() {
                            update_memory_view(weak, &emu_next, *start, WINDOW_SIZE_BYTES);
                        }
                    });
                }

                // Callback for handling the memory "go to location" field
                {
                    let emu_goto = emu_for_memory.clone();
                    let window_goto = window_start_addr_for_memory.clone();
                    let memory_weak_goto = memory_weak_clone.clone();
                    let mem_len = emu_goto.borrow().bus.memory().get_data().len();
                    
                    new_win.global::<AppLogic>().on_goto_location(move |text| {

                        let text = text.trim();

                        if text.is_empty() {
                            println!("Requested memory location is empty. Doing nothing.");
                            return;
                        }

                        let trimmed = text.strip_prefix("0x").unwrap_or(text);

                        match u64::from_str_radix(trimmed, 16) {
                            Ok(addr_u64) => {

                                let addr = addr_u64 as usize;

                                if addr >= mem_len {
                                    println!("Address {:X} out of bounds.", addr);
                                    return;
                                }

                                // Compute page start containing this address
                                let page_start = (addr / WINDOW_SIZE_BYTES) * WINDOW_SIZE_BYTES;

                                {
                                    let mut start = window_goto.borrow_mut();
                                    *start = page_start;
                                }

                                if let Some(weak) = memory_weak_goto.borrow().as_ref() {
                                    update_memory_view(weak, &emu_goto, page_start, WINDOW_SIZE_BYTES);
                                }
                            }

                            Err(_) => {
                                println!("Requested memory location is not valid hex: {:?}", trimmed);
                            }
                        }

                    } );

                }

                *memory_weak_clone.borrow_mut() = Some(new_win.as_weak());
                *slot = Some(new_win);
            }

            slot.as_ref().unwrap().show().unwrap();
        }
    });

    ui.show()?;

    // Force first memory population AFTER UI is alive
    {
        let emu_first = emu.clone();
        let window_first = window_start_addr.clone();
        let start = window_first.borrow();
        if let Some(weak) = memory_weak.borrow().as_ref() {
            update_memory_view(weak, &emu_first, *start, WINDOW_SIZE_BYTES);
        }
    }
    slint::run_event_loop()?;

    Ok(())
}

/// Does what it says on the tin.
fn update_memory_view(
    ui_weak: &slint::Weak<MemoryView>,
    emu: &Rc<RefCell<Emulator>>,
    window_start: usize,
    window_size: usize,
) {
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };

    // The props that are in the actual memory view.
    let mem_data = ui.global::<MemoryViewData>();

    let emu = emu.borrow();
    let memory = emu.bus.memory().get_data();
    let mem_len = memory.len();

    if mem_len == 0 {
        return;
    }

    let start = window_start.min(mem_len - 1);
    let end = (start + window_size).min(mem_len);
    let slice = &memory[start..end]; // What we will actually display / update

    let bytes_per_row = 16;
    let total_rows = (slice.len() + bytes_per_row -1) / bytes_per_row;

    let mut addresses: Vec<SharedString> = Vec::with_capacity(total_rows);
    let mut hex_rows: Vec<ModelRc<SharedString>> = Vec::with_capacity(total_rows);
    let mut ascii_rows: Vec<ModelRc<SharedString>> = Vec::with_capacity(total_rows);

    for row in 0..total_rows {
        let row_start = row * bytes_per_row;
        let row_end = (row_start+bytes_per_row).min(slice.len());
        let row_slice = &slice[row_start..row_end];

        let absolute_addr = start + row_start;
        addresses.push(format!("0x{:04X}", absolute_addr).into());

        // The hex version
        let mut hex: Vec<SharedString> = row_slice
            .iter().map(|b| format!("{:02X}", b).into())
            .collect();

        while hex.len() < bytes_per_row {
            hex.push("".into());
        }

        hex_rows.push(ModelRc::new(VecModel::from(hex)));

        // ASCII version
        let mut ascii: Vec<SharedString> = row_slice
            .iter().map(|&b| {
                let c = b as char;
                if c.is_ascii_graphic() || c == ' ' { c } else { '.' }
            })
            .map(|c| c.to_string().into()).collect();

        while ascii.len() < bytes_per_row {
            ascii.push("".into());
        }

        ascii_rows.push(ModelRc::new(VecModel::from(ascii)));
    }

    // Send into the memory view on the slint side
    mem_data.set_addresses(ModelRc::new(VecModel::from(addresses)));
    mem_data.set_hexBytes(ModelRc::new(VecModel::from(hex_rows)));
    mem_data.set_asciiChars(ModelRc::new(VecModel::from(ascii_rows)));

    // Highlight PC
    let pc = emu.cpu.pc as usize;

    if pc >= start && pc < end {
        let offset = pc - start;
        mem_data.set_pcRow((offset / bytes_per_row) as i32);
        mem_data.set_pcCol((offset % bytes_per_row) as i32);
    } else {
        mem_data.set_pcRow(-1);
        mem_data.set_pcCol(-1);
    }

    mem_data.set_windowStartAddress(start as i32);
    mem_data.set_windowSize(window_size as i32);


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


    let path = format!("resources/roms/8080.rom");
    match load_rom_file(&path) {
        Ok(bytes) => {
            emu.insert_rom(bytes);
        }
        Err(e) => {
            println!("File error: {}", e);
        }
    }
    println!("Inserting ROM and loading...");
    emu.reset()?;

    // emu.load_rom(ROM_TST.to_vec())?;

    Ok(emu)
}

/// Just loads provided filepath into a vec.
fn load_rom_file(path: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
}


fn build_frame_from_vram(emu: &Emulator) -> Image {
    const WIDTH: usize = 224;
    const HEIGHT: usize = 256;
    const VRAM_START: usize = 0x2400;


    let memory = emu.bus.memory().get_data();

    let mut buffer =
        SharedPixelBuffer::<Rgba8Pixel>::new(WIDTH as u32, HEIGHT as u32);

    {
        let pixels = buffer.make_mut_slice();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                // Midway stores video rotated 90°
                let byte_index = VRAM_START + (x * HEIGHT + y) / 8;
                let bit = (memory[byte_index] >> (y % 8)) & 1;

                let color = if bit == 1 {
                    Rgba8Pixel::new(255, 255, 255, 255)
                } else {
                    Rgba8Pixel::new(0, 0, 0, 255)
                };

                // Slint buffer is row-major
                // let idx = y * WIDTH + x;
                let idx = (HEIGHT - 1 - y) * WIDTH + x;
                pixels[idx] = color;
            }
        }
    }


    Image::from_rgba8(buffer)
}

fn handle_input(hw: &Rc<RefCell<MidwayHardware>>, input: MidwayInput, pressed: bool) {
    use MidwayInput::*;
    let mut hw = hw.borrow_mut();
    match input {
        Coin    => if pressed { hw.set_bit(0, 0) } else { hw.clear_bit(0, 0) },
        Start1  => if pressed { hw.set_bit(0, 1) } else { hw.clear_bit(0, 1) },
        Start2  => if pressed { hw.set_bit(0, 2) } else { hw.clear_bit(0, 2) },
        Left    => if pressed { hw.set_bit(1, 0) } else { hw.clear_bit(1, 0) },
        Right   => if pressed { hw.set_bit(1, 1) } else { hw.clear_bit(1, 1) },
        Fire    => if pressed { hw.set_bit(1, 2) } else { hw.clear_bit(1, 2) },
        Tilt    => if pressed { println!("TILT!") }
    }
}

fn slint_key_to_midway_input(key: &str) -> Option<MidwayInput> {
    use MidwayInput::*;
    match key {
        "c" => Some(Coin),
        "1" => Some(Start1),
        "2" => Some(Start2),
        "\u{f702}" => Some(Left),
        "ArrowLeft" => Some(Left),
        "ArrowRight" => Some(Right),
        "\u{f703}" => Some(Right),
        " " => Some(Fire),
        _ => None,
    }
}