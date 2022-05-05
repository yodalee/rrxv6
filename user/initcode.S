# Initial process that execs /init.
# This code runs in user space.

# Currently only do a infinite loop
.globl start
start:
loop:
	j loop
