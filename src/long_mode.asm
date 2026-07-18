global long_mode_start

section .text
bits 64
long_mode_start:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

	extern kernel_main

    xor rdi, rdi
    mov edi, ebx

	call kernel_main
.hang:	hlt
	jmp .hang
.end:
