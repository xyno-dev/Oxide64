MAGIC    equ  0xe85250d6        
ARCH     equ 0

section .multiboot2
align 8

header_start:
	dd MAGIC
	dd ARCH
	dd header_end - header_start
    dd 0x100000000 - (MAGIC + ARCH + (header_end - header_start))
    align 8
        dw 5                               
        dw 0                               
        dd 20                              
        dd 1024                            
        dd 768                             
        dd 32
    align 8
        dw 0                               
        dw 0                               
        dd 8
header_end:

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table_0:
    resb 4096
p2_table_1:
    resb 4096
p2_table_2:
    resb 4096
p2_table_3:
    resb 4096
stack_bottom:
    resb 4096 * 16
stack_top:

section .rodata
gdt64:
    dq 0 
.code: equ $ - gdt64 
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) 
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text
bits 32

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

check_multiboot:
    cmp eax, 0x36D76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    
    

    
    pushfd
    pop eax

    
    mov ecx, eax

    
    xor eax, 1 << 21

    
    push eax
    popfd

    
    pushfd
    pop eax

    
    
    push ecx
    popfd

    
    
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    
    mov eax, 0x80000000    
    cpuid                  
    cmp eax, 0x80000001    
    jb .no_long_mode       

    
    mov eax, 0x80000001    
    cpuid                  
    test edx, 1 << 29      
    jz .no_long_mode       
    ret
.no_long_mode:
    mov al, "2"
    jmp error

set_up_page_tables:
    mov eax, p3_table
    or eax, 0b11
    mov [p4_table], eax

    ; p3 -> four p2 tables, one per GB
    mov eax, p2_table_0
    or eax, 0b11
    mov [p3_table], eax

    mov eax, p2_table_1
    or eax, 0b11
    mov [p3_table + 8], eax

    mov eax, p2_table_2
    or eax, 0b11
    mov [p3_table + 16], eax

    mov eax, p2_table_3
    or eax, 0b11
    mov [p3_table + 24], eax

    ; map each p2 table: 512 entries * 2MB = 1GB each
    mov ecx, 0
.map_p2_0:
    mov eax, 0x200000
    mul ecx
    or eax, 0b10000011
    mov [p2_table_0 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_0

    mov ecx, 0
.map_p2_1:
    mov eax, 0x200000
    mul ecx
    add eax, 0x40000000        ; offset by 1GB
    or eax, 0b10000011
    mov [p2_table_1 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_1

    mov ecx, 0
.map_p2_2:
    mov eax, 0x200000
    mul ecx
    add eax, 0x80000000        ; offset by 2GB
    or eax, 0b10000011
    mov [p2_table_2 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_2

    mov ecx, 0
.map_p2_3:
    mov eax, 0x200000
    mul ecx
    add eax, 0xC0000000        ; offset by 3GB
    or eax, 0b10000011
    mov [p2_table_3 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_3

    ret

enable_paging:
    
    mov eax, p4_table
    mov cr3, eax

    
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

global _start
extern long_mode_start
_start:
	mov esp, stack_top
    push ebx
    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables 
    call enable_paging

    lgdt [gdt64.pointer]
    pop ebx
    jmp gdt64.code:long_mode_start

