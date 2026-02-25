        ORG 0000h

START:
        LXI H, 8000h            ; Just a place to store coin cointer
        MVI A, 00h
        STA LATCH               ; latch = 0

MAIN_LOOP:
        IN  00h                 ; read input port 0
        ANI 01h                 ; isolate coin bit (bit 0)

        JNZ COIN_UP             ; if bit == 1, coin released

; --- coin bit is LOW (pressed) ---
        LDA LATCH
        ORA A
        JNZ SKIP_INC            ; if already latched, skip

        ; increment credit
        MOV A, M
        INR A
        MOV M, A

        ; set latch
        MVI A, 01h
        STA LATCH

SKIP_INC:
        JMP MAIN_LOOP

; --- coin released ---
COIN_UP:
        MVI A, 00h
        STA LATCH               ; clear latch
        JMP MAIN_LOOP

; --- data ---
LATCH:  DB 00h
