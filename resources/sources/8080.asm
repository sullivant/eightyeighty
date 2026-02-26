        ORG 0000H

; Just writes '8080' to the location where video ram starts.
MAIN:
        LXI H, 2000H        ; HL = video start

        MVI M, '8'          ; Write 8
        INX H
        MVI M, '0'          ; Write 8
        INX H
        MVI M, '8'          ; Write 8
        INX H
        MVI M, '0'          ; Write 8
        INX H

LOOP:
        HLT                     ; Halt
        JMP     LOOP            ; Keep spinning

