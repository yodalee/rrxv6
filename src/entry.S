.section .text.entry
.global _entry
.equ STACK_SIZE, 4096
_entry:
	# set up a stack for C.
	# STACK0 is declared in main.rs,
	# with a 4096-byte stack per CPU.
	# sp = STACK0 + (hartid * 4096)
	la sp, STACK0
	li a0, STACK_SIZE
	csrr a1, mhartid
	addi a1, a1, 1
	mul a0, a0, a1
	add sp, sp, a0

	# jump to start() in start.c
	call start

spin:
	j spin
