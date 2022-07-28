rrxv6
=====

rust riscv xv6 implementation

# How To Build?
1. Use rustup to install the target `riscv64imac-unknown-none-elf`  
`rustup target install riscv64imac-unknown-none-elf`
2. Install the riscv64 gcc: `riscv64-unknown-elf-gcc` using your linux package manager 
3. cargo build

# How To Run?
1. Install qemu-system-riscv.
2. Execute:  
`qemu-system-riscv64 -machine virt -bios none -m 128M -smp 1 -nographic
-s -kernel target/riscv64imac-unknown-none-elf/debug/rrxv6`

You should see output `Hello World`

# How To Debug
1. Install `riscv64-elf-gdb`
2. Execute:  
`qemu-system-riscv64 -machine virt -bios none -m 128M -smp 1 -nographic
-S -s -kernel target/riscv64imac-unknown-none-elf/debug/rrxv6`
3. In another terminal, execute:  
`riscv64-elf-gdb -q target/riscv64imac-unknown-none-elf/debug/rrxv6`
