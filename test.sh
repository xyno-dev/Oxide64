#!/usr/bin/env bash

cp "$1" ./isodir/boot/kernel.bin
cp ./src/assets/* ./isodir/boot/assets/
grub2-mkrescue -o oxide64-test.iso isodir
qemu-system-x86_64 -enable-kvm -cdrom oxide64-test.iso -d int -D qemu.log
