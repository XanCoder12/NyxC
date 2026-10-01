pub fn generate_runtime_glue(has_main: bool, has_start: bool) -> String {
    let mut asm = String::new();

    if has_main && !has_start {
        asm.push_str(".globl _start\n");
        asm.push_str("_start:\n");
        asm.push_str("    call main\n");
        asm.push_str("    mov ebx, eax\n");
        asm.push_str("    mov eax, 1\n"); // SYS_EXIT
        asm.push_str("    int 0x80\n\n");
    }

    // Built-in NyxaraOS syscall implementations (cdecl ABI)
    asm.push_str(".globl sys_exit\n");
    asm.push_str("sys_exit:\n");
    asm.push_str("    mov ebx, [esp + 4]\n");
    asm.push_str("    mov eax, 1\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_write\n");
    asm.push_str("sys_write:\n");
    asm.push_str("    push ebx\n");
    asm.push_str("    mov ebx, [esp + 8]\n");
    asm.push_str("    mov ecx, [esp + 12]\n");
    asm.push_str("    mov edx, [esp + 16]\n");
    asm.push_str("    mov eax, 4\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    pop ebx\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_read\n");
    asm.push_str("sys_read:\n");
    asm.push_str("    push ebx\n");
    asm.push_str("    mov ebx, [esp + 8]\n");
    asm.push_str("    mov ecx, [esp + 12]\n");
    asm.push_str("    mov edx, [esp + 16]\n");
    asm.push_str("    mov eax, 3\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    pop ebx\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_fork\n");
    asm.push_str("sys_fork:\n");
    asm.push_str("    mov eax, 2\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_waitpid\n");
    asm.push_str("sys_waitpid:\n");
    asm.push_str("    push ebx\n");
    asm.push_str("    mov ebx, [esp + 8]\n");
    asm.push_str("    mov ecx, [esp + 12]\n");
    asm.push_str("    mov edx, [esp + 16]\n");
    asm.push_str("    mov eax, 7\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    pop ebx\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_execve\n");
    asm.push_str("sys_execve:\n");
    asm.push_str("    push ebx\n");
    asm.push_str("    mov ebx, [esp + 8]\n");
    asm.push_str("    mov ecx, [esp + 12]\n");
    asm.push_str("    mov edx, [esp + 16]\n");
    asm.push_str("    mov eax, 11\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    pop ebx\n");
    asm.push_str("    ret\n\n");

    asm.push_str(".globl sys_getpid\n");
    asm.push_str("sys_getpid:\n");
    asm.push_str("    mov eax, 20\n");
    asm.push_str("    int 0x80\n");
    asm.push_str("    ret\n\n");

    asm
}
