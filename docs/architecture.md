# NyxC Compiler Architecture

## Overview
NyxC is an ahead-of-time (AOT) systems programming language compiler targeting **NyxaraOS**, an x86 32-bit (i386 Protected Mode) hybrid Unix-like operating system.

## Compilation Pipeline

```
[ NyxC Source (.nyx) ]
        │
        ▼ (Lexer)
   [ Tokens ] (Span, Keywords, Literals, Operators, Newlines)
        │
        ▼ (Parser)
   [ Abstract Syntax Tree (AST) ] (Items, Functions, Stmts, Exprs)
        │
        ▼ (Semantic Analyzer & Type Checker)
   [ Validated Typed AST & Symbol Tables ]
        │
        ▼ (Codegen: x86_gas)
   [ x86-32 Assembly (Intel Syntax) ] (.section .text, .rodata, _start)
        │
        ▼ (GNU Assembler: as --32)
   [ ELF32 Object File (.o) ]
        │
        ▼ (GNU Linker: ld -m elf_i386 -T runtime/user.ld -nostdlib)
   [ NyxaraOS Userland Executable (.elf) ] (Base 0x04000000)
```

## Target Specification
- **Architecture**: `i386` (x86 32-bit Protected Mode).
- **Binary Format**: ELF32 LSB executable (`elf32-i386`).
- **Load Address**: `0x04000000` (64 MB virtual address in user space).
- **Entry Point**: `_start` (auto-generated runtime wrapper calling `main` and issuing `SYS_EXIT`).
- **Syscall Mechanism**: Trap gate `int 0x80`.
  - `eax`: Syscall number.
  - `ebx`: Argument 1.
  - `ecx`: Argument 2.
  - `edx`: Argument 3.
  - `esi`: Argument 4.
  - Return value returned in `eax`.

## Calling Convention & Stack Layout
NyxC adheres to standard 32-bit `cdecl` calling convention:
- Arguments are pushed right-to-left onto the stack.
- Return values are passed in `eax`.
- Caller cleans up stack arguments via `add esp, 4 * N`.
- Stack frames use standard prologue/epilogue:
  ```assembly
  push ebp
  mov ebp, esp
  sub esp, 256
  ...
  mov esp, ebp
  pop ebp
  ret
  ```

## Module Hierarchy
- `src/token.rs`: Token definitions and source location spans.
- `src/lexer.rs`: Lexical scanner with comment, escape, and newline handling.
- `src/ast.rs`: AST nodes for types, expressions, statements, and items.
- `src/parser.rs`: Precedence-based recursive descent parser.
- `src/sema/`: Type checker, symbol tables, and scope validation.
- `src/codegen/`: Assembly generator producing GNU `as` Intel syntax (`.intel_syntax noprefix`).
- `src/driver.rs`: Pipeline orchestrator calling assembler and linker.
- `runtime/user.ld`: Linker script matching NyxaraOS userland layout.
- `lib/nyx/`: Standard library headers and syscall bindings.
