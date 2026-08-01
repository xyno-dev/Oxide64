<h1 align="center">
  <a href="https://github.com/xyno-dev/Oxide64">
    <img
      src="https://github.com/xyno-dev/Oxide64/blob/3979a054305dfc130a523d245fe0836031ef61a5/src/assets/logo-transparent.svg"
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

The reason why Oxide64 uses make rather than cargo's build tool is because
it's easier to use my linker script when linking manually with ld and
cargo's build tool (afaik) cannot assemble my assembly for me.

To build Oxide64, run `make` in the project's root directory.

## Running

Make can be used to run a QEMU/KVM virtual machine for Oxide64.
Whether or not you are using the Make command, there are prerequisites
you must meet before running the kernel:

- A master disk connected through the virtual PATA/IDE interface formatted
with the DOS/MBR partitoning scheme and FAT16 file system

To run Oxide64, run `make run` in the project's root directory.

## Flashing

I would personally recommend using your own flashing tool to flash the
resulting `oxide64.iso`, but I do have a `make flash` target set up to
flash the iso to `/dev/sda` which is usually what my personal USB
drive shows up as.

## Cleaning

Make is once again used to clean all unnecessary build garbage.

To clean all unnecessary files, run `make clean` in the project's root directory.
