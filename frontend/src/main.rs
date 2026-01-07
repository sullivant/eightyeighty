slint::include_modules!();

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

// Allows for integration with the running system as if it was MidwayHardware.
use emulator::bus::IoDevice;
use emulator::devices::hardware::midway::MidwayHardware;
use emulator::{self, Emulator, RunState};
use slint::ToSharedString;

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

    // A timer that allows periodic syncing
    let ui_weak_sync = ui.as_weak();
    let sync_timer = slint::Timer::default();
    sync_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(ui) = ui_weak_sync.upgrade() {
                ui.global::<AppLogic>().invoke_sync();
            }
        },
    );


    // Handle syncing everything at once.
    ui.global::<AppLogic>().on_sync(move || {
        let Some(ui) = ui_weak_all.upgrade() else {
            return;
        };

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
    });



    // Handle the request to cleanly exit the app
    ui.global::<AppLogic>().on_cb_exit(|| slint::quit_event_loop().unwrap() );
    ui.global::<AppLogic>().on_cb_show_settings(|| println!("Showing settings...") );

    ui.show()?;
    slint::run_event_loop()?;

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

