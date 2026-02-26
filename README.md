# eightyeighty
An Intel 8080 Emulator, written in Rust.




### Making a ROM file from hex:
`xxd -r -p ./test.hex test.rom`

### Making a ROM file from asm:
`asm80 resources/roms/8080.asm -o resources/roms/8080.rom`

### Making a BIN(ROM) file from asm8080:
`asm8080 resources/roms/8080.asm -oresources/roms/8080`
See: https://github.com/begoon/asm8080


