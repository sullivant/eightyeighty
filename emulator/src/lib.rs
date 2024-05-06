#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;
use constants::{VRAM_SIZE, VRAM_START};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console::{self};
use cpu::instructions::{self, Instruction};
use std::f64;

/**
 * This library is, at its heart, simply a WASM bound wrapper to the calls necessary to do
 * stuff within main.rs and cpu.rs.
 */

#[derive(Clone)]
pub struct Emulator {
    cpu: CPU,
}

impl Emulator {
    const fn new() -> Emulator {
        Emulator { cpu: CPU::new() }
    }
}

// Our actual emulator is really just a wrapped call to CPU
static mut EMULATOR: Emulator = Emulator::new();

#[wasm_bindgen]
#[must_use]
extern "C" {
    fn alert(s: String);

    // Various forms of logging with different signatures
    // Thank you  wasm-bindgen guide.  Of course we can also use
    // console::log_2(...)
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
#[no_mangle]
pub fn cpu_set_disassemble(flag: bool) {
    console::log_2(&"Setting disassemble flag to:".into(), &flag.into());
    unsafe {
        EMULATOR.cpu.disassemble(flag);
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_get_disassemble() -> bool {
    unsafe { EMULATOR.cpu.disassemble }
}

/// Since we cannot directly open a local file in Web Assembly, we need to expect
/// the ROM data to come from the JavaScript side, on the browser.  So let's have
/// a function here that will simply allow for the mapping of an array of data
/// into the ROM space in memory.
#[wasm_bindgen]
#[no_mangle]
pub fn cpu_memory_write(location: usize, data: u8) -> Result<bool, JsValue> {
    unsafe {
        match EMULATOR.cpu.memory().write(location, data) {
            Ok(_) => (),
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
            }
        };
    }

    Ok(true)
}

/// This returns a 16 byte slice of memory, based off of a starting
/// address and consists of an array that is formatted in address/value pairs like this: [[0,255], [1,128]]
#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_get_memory(start: usize) -> String {
    unsafe {
        format!("{:?}",EMULATOR.cpu.memory.get_slice(start))
    }
}

/// Returns an array containing all of the current register values as well as PC.
#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_registers() -> String {
    let mut ret: [usize; 9] = [0; 9];

    unsafe {
        let regs = EMULATOR.cpu.get_all_registers();
        // (&self.pc, &self.sp, &self.a, &self.b, &self.c, &self.d, &self.e, &self.h, &self.l)
        ret[0] = *regs.0; // PC
        ret[1] = *regs.1 as usize; // SP
        ret[2] = *regs.2 as usize; // A
        ret[3] = *regs.3 as usize; // B
        ret[4] = *regs.4 as usize; // C
        ret[5] = *regs.5 as usize; // D
        ret[6] = *regs.6 as usize; // E
        ret[7] = *regs.7 as usize; // H
        ret[8] = *regs.8 as usize; // L
    
        format!("{:?}", ret)
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_state() -> String {
    unsafe { 
        EMULATOR.cpu.to_string() 
    }
}


#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_instructions() -> JsValue {
    unsafe {
        let mut ret = Vec::with_capacity(2);
        ret.push(EMULATOR.cpu.current_instruction);
        ret.push(EMULATOR.cpu.next_instruction);
        serde_wasm_bindgen::to_value(&ret).unwrap()
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_tick() -> u8 {
    unsafe {
        match EMULATOR.cpu.tick() {
            Ok(v) => v,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                0
            }
        }
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_reset() -> bool {
    unsafe {
        match EMULATOR.cpu.reset() {
            Ok(_) => true,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                false
            }
        }
    }
}

#[wasm_bindgen]
#[no_mangle]
pub fn cpu_get_vram() -> String {
    unsafe {
        format!("{:?}",EMULATOR.cpu.memory.get_vram())
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn vram_update() {  
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.save();
    context.translate(0.0, 128.0).unwrap();
    context.rotate(270.0*f64::consts::PI/180.0).unwrap();

    context.set_fill_style(&JsValue::from("blue".to_string()));
    context.fill_rect(0.00, 0.00, 256.0, 300.0);

    context.set_fill_style(&JsValue::from("white".to_string()));

    unsafe {
        let mut pixel: u8 = 0;
        let mut rectX: f64 = 0.0;
        let mut rectY: f64 = 0.0;

        // console::log_1(&JsValue::from(EMULATOR.cpu.memory.read(VRAM_START).unwrap().to_string()));
        // EMULATOR.cpu.memory.write(VRAM_START as usize, 0xF).unwrap();
        // EMULATOR.cpu.memory.write(VRAM_START+1 as usize, 0xF).unwrap();
        // console::log_1(&JsValue::from(EMULATOR.cpu.memory.read(VRAM_START).unwrap().to_string()));

        for y in 0 .. 256 {
            for x in 0 .. 32 {
                let loc = y * 32 + x + VRAM_START;
                pixel = EMULATOR.cpu.memory.read(loc as usize).unwrap();
                for b in 0 .. 8 {
                    if (pixel >> b) & 1 > 0 { // 
                        rectX = ((x*8)+b) as f64;
                        rectY = y as f64;
                        context.fill_rect(rectX, rectY, 1.0, 1.0);
                    }
                }
            }
        }
    }

    context.restore();


    // The "smile" example
    // context.begin_path();
    // // Draw the outer circle.
    // context
    //     .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
    //     .unwrap();
    // // Draw the mouth.
    // context.move_to(110.0, 75.0);
    // context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();
    // // Draw the left eye.
    // context.move_to(65.0, 65.0);
    // context
    //     .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
    //     .unwrap();
    // // Draw the right eye.
    // context.move_to(95.0, 65.0);
    // context
    //     .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
    //     .unwrap();
    // context.stroke();    
}


// // An example from 'the book' that shows dom interaction
// // source: https://rustwasm.github.io/docs/book/
// #[wasm_bindgen(start)]
// fn run() -> Result<(), JsValue> {
//     let window = web_sys::window().expect("No global window exists!");
//     let document = window.document().expect("Should have a document on the window.");
//     let body = document.body().expect("Document should have a body.");

//     // Make the element
//     let val = document.create_element("p")?;
//     val.set_text_content(Some("Hello from rust!"));
//     body.append_child(&val)?;
//     Ok(())
// }
