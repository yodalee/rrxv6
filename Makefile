kernel=src
U=user
TOOLPREFIX = riscv64-unknown-elf-
CC = $(TOOLPREFIX)gcc
AS = $(TOOLPREFIX)gas
LD = $(TOOLPREFIX)ld
OBJCOPY = $(TOOLPREFIX)objcopy
OBJDUMP = $(TOOLPREFIX)objdump
GDB = riscv64-elf-gdb
CFLAGS = -Wall -Werror -O -fno-omit-frame-pointer -ggdb -gdwarf-2

$U/initcode: $U/initcode.S
	$(CC) $(CFLAGS) -march=rv64g -nostdinc -I. -Ikernel -c $U/initcode.S -o $U/initcode.o
	$(LD) $(LDFLAGS) -N -e start -Ttext 0 -o $U/initcode.out $U/initcode.o
	$(OBJCOPY) -S -O binary $U/initcode.out $U/initcode
	$(OBJDUMP) -S $U/initcode.o > $U/initcode.asm

QEMU = qemu-system-riscv64
CPUS := 1
KERNEL = target/riscv64imac-unknown-none-elf/debug/rrxv6

QEMUOPTS = -machine virt -bios none -m 128M -smp $(CPUS) -nographic
# use virtio v1.0
QEMUOPTS += -global virtio-mmio.force-legacy=false
QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMUOPTS += -kernel ${KERNEL}
qemu:
	${QEMU} ${QEMUOPTS}

qemu_debug:
	@echo "Run: 'riscv64-elf-gdb -q ${KERNEL}' in another terminal"
	${QEMU} -S -s ${QEMUOPTS}

gdb:
	${GDB} -q ${KERNEL}
