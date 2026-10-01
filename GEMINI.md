# GEMINI.md - NyxC Compiler Guide

## Project Overview
NyxC is a lightweight, clean systems programming language compiler specifically designed for **NyxaraOS** (`/home/akrom/Projects/NyxaraOS`).
- **Host Compiler**: Rust (pure standard library, zero external crate dependencies).
- **Target Architecture**: x86 32-bit (i386 Protected Mode).
- **Target OS**: NyxaraOS userland (ELF32, entry point `0x04000000`, `int 0x80` syscall ABI).
- **Standard Library**: `lib/nyx/` (`sys.nyx`).

## Strict Code Style & AI Rules
- **No AI Slop / Over-commenting**: Do not add redundant comments. Explain non-obvious algorithms or register layouts only.
- **No Heavy ASCII Separators**: Never use long divider lines (`// ==========================`, `/* ----------------- */`, `##########`, etc.). Keep code clean and minimalistic.
- **Modular Code**: Never dump massive amounts of code into a single file. Keep clear separation of concerns across files and folders.
- **Git Commit Policy**: Never run `git commit` directly. Always output proposed commit commands for each file created/modified:
  ```bash
  git add <file>
  git commit -m "<concise message in English>"
  ```
  Provide one commit per file, except for documentation updates inside `/docs`.
- **Documentation**: Keep documentation in `/docs` in sync whenever new features or changes are introduced.

## Directory Structure
- `src/`
  - `main.rs`: CLI interface (`nyxc build`, `check`, `emit-asm`).
  - `lib.rs`: Library crate root.
  - `token.rs`: Token definitions and source spans.
  - `lexer.rs`: Lexical analyzer with newline and escape parsing.
  - `ast.rs`: Abstract Syntax Tree definitions.
  - `parser.rs`: Recursive descent parser with optional semicolons.
  - `sema/`: Semantic analysis, type checking (`types.rs`), and symbol resolution (`symbols.rs`).
  - `codegen/`: Code generators (`x86_gas.rs`) and runtime glue (`runtime.rs`).
  - `driver.rs`: Pipeline driver calling `as --32` and `ld -m elf_i386`.
  - `target.rs`: Target configurations (`nyxara-x86`, `linux-x86`).
- `runtime/`
  - `user.ld`: ELF32 linker script for NyxaraOS userland (`0x04000000`).
- `lib/`
  - `nyx/sys.nyx`: Standard library syscall declarations and wrappers.
- `docs/`
  - `architecture.md`: Compiler architecture and pipeline design.
  - `language-spec.md`: Formal language specification.
  - `nyxara-integration.md`: Instructions for running NyxC programs on NyxaraOS.
- `tests/`
  - `compile_tests.rs`: Integration and end-to-end tests.

## Build & Test Commands
- Build compiler: `cargo build` (debug) or `cargo build --release` (release)
- Run tests: `cargo test`
- Compile NyxC program to ELF32: `cargo run -- build <file.nyx> -o <file.elf>`
- Check syntax and types: `cargo run -- check <file.nyx>`
- Emit x86-32 assembly: `cargo run -- emit-asm <file.nyx>`
