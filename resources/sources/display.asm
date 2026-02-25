ORG 0000h

        ; === TOP-LEFT: origin (0,0), VRAM row offset = 0 ===
        LXI H, 2400h       ; Start of VRAM (row 0, byte 0)
        MVI B, 16          ; 16 rows 
TL_LOOP:
        MVI A, 0FFh        ;
        MOV M, A           ; Write byte 0 of this row
        INX H              ; Increment HL by 1
        MOV M, A           ; Write byte 1 of this row (pixels 8-15)
        LXI D, 001Fh       ; Advance remaining 30 bytes to next row
        DAD D
        DCR B
        JNZ TL_LOOP        ; Will jump to TL_LOOP if B is nonzero.


        ; === TOP-RIGHT: x=208, y=0 (column 208, byte 0) ===
        ; Row start = 2400h + 208*32 = 2400h + 1A00h = 3E00h
        LXI H, 3E00h       ; 
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

HLT