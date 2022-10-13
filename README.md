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
- POP
- PUSH
- XHTL
- SPHL
- XCHG
### 16 Bit Arithmetic / Logic
- DAD
### 8 Bit Load / Store / Move
- STA
- MVI
- LDAX
- LDA
- MOV
### 8 Bit Arithmetic / Logic
- INR
- DCR
- STC
- CMA
- CMC
- SUB
- SBB
- SUI
- SBI
- XRI


