multiboot-kernel.iso : kernel.bin
	cp ./src/assets/* ./isodir/boot/assets/
	grub2-mkrescue -o multiboot-kernel.iso isodir

libmultiboot_kernel.a:
	cargo build --target x86_64-unknown-none --release

boot.o:
	nasm -felf64 ./src/boot.asm

long_mode.o:
	nasm -felf64 ./src/long_mode.asm

kernel.bin : boot.o long_mode.o libmultiboot_kernel.a
	ld -T linker.ld -o kernel.bin ./src/boot.o ./src/long_mode.o ./target/x86_64-unknown-none/release/libmultiboot_kernel.a
	cp kernel.bin ./isodir/boot/kernel.bin

run : multiboot-kernel.iso
	qemu-system-x86_64 -enable-kvm -cdrom multiboot-kernel.iso -bios /usr/share/edk2/ovmf/OVMF_CODE.fd -d int -D qemu.log

flash : multiboot-kernel.iso
	sudo dd if=multiboot-kernel.iso of=/dev/sda

clean:
	rm -rf ./src/boot.o ./src/long_mode.o \
		./kernel.bin ./multiboot-kernel.iso \
		./qemu.log ./target ./isodir/boot/assets/*

