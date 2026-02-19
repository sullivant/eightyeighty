ORG 0000h

        ; === TOP-LEFT: origin (0,0), VRAM row offset = 0 ===
        LXI H, 2400h       ; Start of VRAM (row 0, byte 0)
        MVI B, 16          ; 16 rows
TL_LOOP:
        MVI A, 0FFh
        MOV M, A           ; Write byte 0 of this row
        INX H
        MOV M, A           ; Write byte 1 of this row (pixels 8-15)
        LXI D, 001Fh       ; Advance remaining 30 bytes to next row
        DAD D
        DCR B
        JNZ TL_LOOP

        ; === TOP-RIGHT: origin (240,0), VRAM row byte = 30 (240/8=30) ===
        ; Row start = 2400h + 30 = 241Eh, write bytes 30 and 31
        LXI H, 241Eh       ; 2400h + 1Eh
        MVI B, 16
TR_LOOP:
        MVI A, 0FFh
        MOV M, A           ; Byte 30
        INX H
        MOV M, A           ; Byte 31
        LXI D, 001Fh       ; Skip 31 bytes to next row start byte 30
        DAD D
        DCR B
        JNZ TR_LOOP

        ; === BOTTOM-LEFT: origin (0,240), row 240 = 2400h + 240*32 = 2400h + 1E00h = 4200h ===
        LXI H, 3E00h
        MVI B, 16
BL_LOOP:
        MVI A, 0FFh
        MOV M, A
        INX H
        MOV M, A
        LXI D, 001Fh
        DAD D
        DCR B
        JNZ BL_LOOP

        ; === BOTTOM-RIGHT: row 240, byte 30 = 4200h + 1Eh = 421Eh ===
        LXI H, 3E1Eh
        MVI B, 16
BR_LOOP:
        MVI A, 0FFh
        MOV M, A
        INX H
        MOV M, A
        LXI D, 001Fh
        DAD D
        DCR B
        JNZ BR_LOOP

        HLT