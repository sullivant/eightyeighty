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
