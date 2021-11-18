set confirm off
set architecture riscv:rv64
target remote :1234
symbol-file target/riscv64imac-unknown-none-elf/debug/rrxv6
