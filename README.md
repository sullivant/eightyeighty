# eightyeighty
An Intel 8080 Emulator, written in Rust.




### Making a ROM file from hex:
`xxd -r -p ./test.hex test.rom`

### Making a ROM file from asm:
`asm80 resources/roms/8080.asm -o resources/roms/8080.rom`