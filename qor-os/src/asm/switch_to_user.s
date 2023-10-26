.section .text

.option norvc

.global switch_to_user
switch_to_user:
    csrw mscratch, a0
    li t0, (1 << 7) | (1 << 5) | (1 << 13)
    csrw mstatus, t0
    csrw mepc, a1
    csrw satp, a2

    li t1, 0xaaa
    csrw mie, t1

    la t2, asm_trap_vector
    csrw mtvec, t2

    sfence.vma
    mv t6, a0

    .set i, 0
    .rept 32
        load_fp %i
        .set i, i + 1
    .endr

    .set i, 1
    .rept 31
        load_gp %i, t6
        .set i, i + 1
    .endr

    mret