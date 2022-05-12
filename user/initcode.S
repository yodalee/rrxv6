# Initial process that execs /init.
# This code runs in user space.

# Currently only do a infinite loop
.globl start
start:
  li a0, 0
  li a1, 'h'
  li a7, 0
  ecall
loop:
  j loop
