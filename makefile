multiboot-kernel.iso : kernel.bin
	cp ./src/assets/* ./isodir/boot/assets/
	grub2-mkrescue -o oxide64.iso isodir

liboxide64.a:
	cargo build --target x86_64-unknown-none --release

boot.o:
	nasm -felf64 ./src/boot.asm

long_mode.o:
	nasm -felf64 ./src/long_mode.asm

kernel.bin : boot.o long_mode.o liboxide64.a
	ld -T linker.ld -o kernel.bin ./src/boot.o ./src/long_mode.o ./target/x86_64-unknown-none/release/liboxide64.a
	cp kernel.bin ./isodir/boot/kernel.bin

run : oxide64.iso
	qemu-system-x86_64 -enable-kvm -cdrom oxide64.iso -bios /usr/share/edk2/ovmf/OVMF_CODE.fd -d int -D qemu.log

flash : oxide64.iso
	sudo dd if=oxide64.iso of=/dev/sda

clean:
	rm -rf ./src/boot.o ./src/long_mode.o \
		./kernel.bin ./oxide64.iso \
		./qemu.log ./target ./isodir/boot/assets/* \
		./isodir/boot/kernel.bin

