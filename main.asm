global _start
section .text

nnv_hlpr_exit_fn:
    push rbp
    mov rbp, rsp

    mov rax, 60
    syscall

    mov rsp, rbp
    pop rbp
    ret

_start:
    ; exit statement with code '30'
    mov rdi, 30
    call nnv_hlpr_exit_fn

    mov rax, 60
    xor rdi, rdi
    syscall
