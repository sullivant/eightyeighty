# eightyeighty
An Intel 8080 Emulator, written in Rust.

[![Continuous Integration](https://github.com/sullivant/eightyeighty/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sullivant/eightyeighty/actions/workflows/ci.yml)

## The deal
- Clone.
- Ensure lib sdl2 installed
- `cargo build`
- Acquire ROMs and place into ./resources/roms/
- `cargo run -- ROMNAME`
- Laugh as it fails because I messed up an opcode.  

## Opcode Categories
### General / Control
- IN
- DI
- EI
### Jumps / Calls
- JPO
- JP
- RST
- RZ
- PCHL
- JC
- JPE
### 16 Bit Load / Store / Move
- LXI
- SHLD
- LHLD
- XHTL
- SPHL
### 8 Bit Load / Store / Move
- STA
- LDAX
- MOV
### 8 Bit Arithmetic / Logic
- INR
- SBB
- SUI
- SBI
- XRI


