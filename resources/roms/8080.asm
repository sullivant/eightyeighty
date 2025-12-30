ORG 0000h

; Just writes '8080' to the location where video ram starts.

LXI H, 8000h        ; HL = video start

MVI M, '8'          ; Write 8
INX H
MVI M, '0'          ; Write 8
INX H
MVI M, '8'          ; Write 8
INX H
MVI M, '0'          ; Write 8
INX H

HLT
