<h1 align="center">
  <a href="https://github.com/xyno-dev/Oxide64">
    <img
      src="https://github.com/xyno-dev/Oxide64/blob/3979a054305dfc130a523d245fe0836031ef61a5/src/assets/logo-transparent.png"
      alt="Logo" width="512" height="512">
  </a>
  <br />
  Oxide64
</h1>

## Overview

Oxide64 is a small toy kernel written in Rust which isn't meant to have any real functionality.
All it's supposed to be is a kernel which I can mess around with while learning many low-level
concepts.

## Building
> [!TIP]
> ### Prerequisites
> Before building Oxide64, you will need the following tools installed:
> - `cargo`
> - `grub2-mkrescue` (is under different packages on different Linux distributions)
> - `xorriso`

> [!IMPORTANT]
> On some Linux distributions, the  `grub2-mkrescue` binary is called `grub-mkrescue`.
> They both correspond to the same grub2 binary. If this is the case on your distro,
> modify the Makefile to use `grub-mkrescue` rather than `grub2-mkrescue`.

> [!NOTE]
> The reason why Oxide64 uses make rather than cargo's build tool is because
> it's easier to use my linker script when linking manually with ld and
> cargo's build tool cannot assemble my assembly for me before linking with
> the resulting Rust binary.

To build Oxide64, run `make` in the project's root directory.

## Running
> [!TIP]
> ### Prerequisites
> Before running Oxide64, you will need a disk image formatted with the FAT16 file system.
> To create a disk image and format it as FAT16, run:
> ```
> qemu-img create -f raw disk.img 256M
> mkfs.fat -F16 disk.img
> ```
> You will also need QEMU and KVM set up. If you would not like to use KVM,
> modify the Makefile and remove the `-enable-kvm` flag in the QEMU command.

Make can also be used to run a QEMU/KVM virtual machine for Oxide64.

To run Oxide64, run `make run` in the project's root directory.

## Flashing
> [!IMPORTANT]
> Oxide64 cannot run on modern hardware! It currently uses many older hardware features, such as:
> - ATA PIO
> - 8259 PIC
> - 8253/8254 PIT
> - PS/2 Keyboard
> 
> This is because implementing drivers for these older hardware features is much simpler and easier
> than implementing drivers for their modern replacements (such as AHCI, APIC, APIC Timer and xHCI).

I would personally recommend using your own flashing tool to flash the
resulting `oxide64.iso`, but I do have a `make flash` target set up to
flash the iso to `/dev/sda` which is usually what my personal USB
drive shows up as.

## Cleaning

Make is once again used to clean all unnecessary build garbage.

To clean all unnecessary files, run `make clean` in the project's root directory.
