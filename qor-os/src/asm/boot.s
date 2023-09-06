# Do not produce compressed instructions
.option norvc

# Section which will be placed as 0x8000_0000 (The start location for qemu)
.section .text.init
.global _start
_start:
    j _start