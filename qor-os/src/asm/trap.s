.section .text

.option norvc

.altmacro
.set NUM_GP_REGS, 32
.set NUM_FP_REGS, 32
.set REG_SIZE, 8
.set MAX_CPUS, 8

.macro save_gp i, basereg=t6
    sd x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.macro load_gp i, basereg=t6
    ld x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.macro save_fp i, basereg=t6
    fsd f\i, ((NUM_GP_REGS + (\i))*REG_SIZE)(\basereg)
.endm

.macro load_fp i, basereg=t6
    fld f\i, ((NUM_GP_REGS + (\i))*REG_SIZE)(\basereg)
.endm

.global asm_trap_vector
asm_trap_vector:
    # The CPU was interrupted
    
    # First we must save all of the registers
    csrrw t6, mscratch, t6

    .set i, 1
    .rept 30
        save_gp %i
        .set i, i + 1
    .endr

    mv t5, t6
    csrr t6, mscratch
    save_gp 31, t5

    csrr t1, mstatus
    srli t0, t1, 13
    andi t0, t0, 3
    li t3, 3
    bne t0, t3, skip_float_save
    .set i, 0
    .rept 32
        save_fp %i, t5
        .set i, i+1
    .endr

skip_float_save:
    csrw mscratch, t5

    # Set up the arguments for the m_trap function
    csrr a0, mepc
    csrr a1, mtval
    csrr a2, mcause
    csrr a3, mhartid
    csrr a4, mstatus
    mv a5, t5
    ld sp, 520(a5)
    csrr a6, satp

    # Call the m_trap function
    call m_trap

    # Restore Registers
    csrw mepc, a0
    csrr t6, mscratch

    .set i, 1
    .rept 31
        load_gp %i
        .set i, i + 1
    .endr
    
    # Jump back to where the interrupt was triggered
    mret
