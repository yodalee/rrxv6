# Initial process that execs /init.
# This code runs in user space.

# Currently only do a infinite loop
.globl start
start:
  li a0, 13
  la a1, helloworld
  li a7, 0
  ecall
loop:
  j loop

.data
helloworld: .ascii "Hello World!"
