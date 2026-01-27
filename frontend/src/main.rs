slint::include_modules!();

use std::cell::RefCell;
use std::{fs, io};
use std::rc::Rc;
use std::time::Duration;
use rfd::FileDialog;

// Allows for integration with the running system as if it was MidwayHardware.
use emulator::bus::IoDevice;
use emulator::devices::hardware::midway::MidwayHardware;
use emulator::{self, Emulator, RunState};
use slint::{ToSharedString, VecModel};
use slint::{ModelRc, SharedString};

// A simple test rom with a few instructions to load at the start
const ROM_TST: &[u8] = &[0x3E, 0x42, 0x76];

const WINDOW_SIZE_BYTES: usize = 256;

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
                        emu.run_blocking(Some(200));
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
    memory_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1), // 1 FPS
        move || {
            let start = *window_start_for_mem.borrow();

            if let Some(weak) = memory_weak_timer.borrow().as_ref() {
                update_memory_view(weak, &emu_for_mem_timer, start, WINDOW_SIZE_BYTES);
            }
        }
    );

    // Handle syncing everything, except memory view portion, at once.
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
            regs.set_pc(cpu.pc as i32);

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
                        println!("PREVIOUS!");
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
                        println!("NEXT!");
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
