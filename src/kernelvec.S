
# ============ MACRO ==================
.macro ctx_save base
        sd ra,  0(\base)
        sd sp,  8(\base)
        sd s0,  16(\base)
        sd s1,  24(\base)
        sd s2,  32(\base)
        sd s3,  40(\base)
        sd s4,  48(\base)
        sd s5,  56(\base)
        sd s6,  64(\base)
        sd s7,  72(\base)
        sd s8,  80(\base)
        sd s9,  88(\base)
        sd s10, 96(\base)
        sd s11, 104(\base)
.endm

.macro ctx_load base
        ld ra,  0(\base)
        ld sp,  8(\base)
        ld s0,  16(\base)
        ld s1,  24(\base)
        ld s2,  32(\base)
        ld s3,  40(\base)
        ld s4,  48(\base)
        ld s5,  56(\base)
        ld s6,  64(\base)
        ld s7,  72(\base)
        ld s8,  80(\base)
        ld s9,  88(\base)
        ld s10, 96(\base)
        ld s11, 104(\base)
.endm
# ============ Macro END   ==================

# Switch context from a0 to a1
# after ctx_load, return address ra will be replaced by a1->ra
.global sys_switch
.option norelax
.align 4
sys_switch:
	ctx_save a0
	ctx_load a1
	ret

# unexpected exception/interrupt happened in M-mode
# use `info registers` in qemu to get CSRs
unexpected_exc:
	j unexpected_exc
unexpected_int:
  j unexpected_int

.globl timervec
.option norelax
.align 4
timervec:
  # start.rs setup memory to be used in mscratch
	# scratch[0, 8, 16] saved register
	# scratch[24] MTIMECMP register
	# scratch[32] interval between interrupts
	csrrw a0, mscratch, a0
	sd a1, 0(a0)
	sd a2, 8(a0)
	sd a3, 16(a0)

  // we should not get exceptions in M-mode.
	// exception will have MSB 0, so it will be positive number
	csrr a1, mcause
	bgez a1, unexpected_exc

  // the only allowed interrupt is machine timer interrupt
	li a2, (1<<63 | 7)
	bne a1, a2, unexpected_int

  # schedule the next timer interrupt
	# by adding interval to mtimecmp.
	ld a1, 24(a0) # CLINT_MTIMECMP(hart)
	ld a2, 32(a0) # interval

	ld a3, 0(a1)
	add a3, a3, a2
	sd a3, 0(a1)

	# raise a supervisor software interrupt.
	li a1, 2
	csrw sip, a1

	ld a3, 16(a0)
	ld a2, 8(a0)
	ld a1, 0(a0)
	csrrw a0, mscratch, a0

	mret
