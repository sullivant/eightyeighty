; ============================================================
; 8080 Interrupt Handler Test ROM
; Syntax: ASM80 (Intel-style hex, H suffix)
; ============================================================
; Tests RST 1 (vector 0008H) and RST 2 (vector 0010H)
;
; RST 1 fired -> writes 01H to memory address 2001H
; RST 2 fired -> writes 01H to memory address 2002H
;
; The main loop enables interrupts and halts/spins so the
; emulator can inject RST 1 and RST 2 at will.
;
; After each handler runs, EI is re-issued before RET because
; the 8080 clears the interrupt-enable flip-flop on interrupt
; entry. If your emulator handles this correctly, subsequent
; interrupts will fire cleanly. If not — you'll find out fast.
; ============================================================

        ORG     0000H

; ============================================================
; RESET VECTOR — execution starts here
; ============================================================
RESET:
        DI
        LXI     H, 2400H    ; load 2400H into HL
        SPHL                ; copy HL into SP
        JMP     MAIN

; ============================================================
; RST 1 HANDLER — vector at 0008H
; ============================================================
        ORG     0008H
        JMP     RST1_HANDLER       ; Handles RST1

; ============================================================
; RST 2 HANDLER — vector at 0010H
; ============================================================
        ORG     0010H
        JMP     RST2_HANDLER       ; Handles RST2


; ============================================================
; MAIN — starts after vectors 
; ============================================================
        ORG     0020H
MAIN:
        ; Clear our test memory locations so we have a known
        ; baseline before any interrupt fires
        LXI     H, 2001H
        MVI     M, 00H          ; Clear RST1 flag
        INX     H
        MVI     M, 00H          ; Clear RST2 flag (2002H)

        EI                      ; Enable interrupts — we're ready

; ============================================================
; SPIN LOOP — just keep looping with interrupts enabled.
; The emulator fires RST 1 or RST 2 externally (e.g. on a
; timer tick, just like Space Invaders' two mid-screen and
; vblank interrupts). We yield with HLT so the emulator has
; a clean instruction boundary to inject the interrupt on,
; then loop back and re-enable.
; ============================================================
LOOP:
        HLT                     ; Halt and wait for interrupt.
                                ; Emulator resumes execution at
                                ; next instruction after servicing.
        EI                      ; Re-enable after HLT resumes
        JMP     LOOP            ; Keep spinning


; ============================================================
; RST Handlers
; ============================================================
        ORG 0030H
RST1_HANDLER:
        PUSH    PSW             ; Save accumulator and flags
        PUSH    H               ; Save HL (we'll use it for MVI M)

        LXI     H, 2001H        ; Point HL at target memory address
        MVI     M, 01H          ; Write 01H to address 2001H

        POP     H               ; Restore HL
        POP     PSW             ; Restore accumulator and flags
        EI                      ; Re-enable interrupts before return
                                ; (8080 clears IFF on interrupt entry)
        RET

RST2_HANDLER:
        PUSH    PSW
        PUSH    H

        LXI     H, 2002H        ; Point HL at target memory address
        MVI     M, 01H          ; Write 01H to address 2002H

        POP     H
        POP     PSW
        EI                      ; Re-enable interrupts before return
        RET



; ============================================================
;    - 2001H should contain 01H (RST1 fired)
;    - 2002H should contain 01H (RST2 fired)
; ============================================================

        END