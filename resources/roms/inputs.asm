        LXI     H, 8000h       ; HL points to memory location 8000h

LOOP:   
        IN      00h            ; Read input port 00h
        ANI     01h            ; Mask all bits except bit 0 (coin 'c' pressed)
        JNZ     NO_COIN        ; If nonzero, coin not pressed, skip increment 

        MOV     A, M           ; Load value at (HL)
        INR     A              ; Increment accumulator
        MOV     M, A           ; Store back to memory at (HL)

NO_COIN:
        JMP     LOOP           ; Repeat forever
