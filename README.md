# NyxC

NyxC adalah compiler bahasa pemrograman tingkat sistem yang dirancang khusus untuk lingkungan userland **NyxaraOS** (sistem operasi x86 32-bit). Compiler ini ditulis menggunakan Rust murni tanpa ketergantungan crate eksternal, menghasilkan berkas assembly GNU x86-32 (`gas`) dan mengompilasinya menjadi file biner ELF32 (`elf32-i386`).

---

## Fitur Utama

- **Zero External Dependencies**: Dibangun dengan Rust standar (edition 2021) tanpa pustaka pihak ketiga.
- **Target Arsitektur**: x86 32-bit (i386 Protected Mode).
- **Target Format**: ELF32 executable (`entry point 0x04000000`, konvensi pemanggilan `cdecl`).
- **Syscall ABI NyxaraOS**: Antarmuka trap gate interrupt `int 0x80` (`eax`=no syscall, `ebx`=arg1, `ecx`=arg2, `edx`=arg3, `esi`=arg4).
- **Type Checking & Mutability**: Mendukung inferensi tipe variabel, konstanta, penegakan mutabilitas (`let` vs `let mut`), pointer, serta tipe primitif.
- **Built-in Functions**:
  - `str_len(s: *u8) -> i32`: Panjang string NUL-terminated (di-inline langsung saat kompilasi).
  - `print(s: *u8)`: Menulis string ke `stdout` tanpa baris baru.
  - `println(s: *u8)`: Menulis string ke `stdout` dengan baris baru (`\n`).
  - `itoa(n: i32) -> *u8`: Konversi integer bertanda menjadi string desimal NUL-terminated.
- **Sintaks Fleksibel**: Titik-koma (`;`) bersifat opsional dan dapat digantikan dengan baris baru (newline).

---

## Tipe Data & Operator

### Tipe Data Primitif
- **Integer Bertanda (Signed)**: `i8`, `i16`, `i32`
- **Integer Tanpa Tanda (Unsigned)**: `u8`, `u16`, `u32`
- **Boolean**: `bool` (`true`, `false`)
- **Pointer**: `*T` (contoh: `*u8`, `*i32`, `**u8`)
- **Void**: `void`

### Operator
- **Aritmatika**: `+`, `-`, `*`, `/`, `%`
- **Perbandingan**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logika & Bitwise**: `&&`, `||`, `!`, `&`, `|`, `^`, `~`, `<<`, `>>`
- **Pointer**: `&var` (address-of), `*ptr` (dereference)

---

## Struktur Direktori

```text
NyxC/
├── src/
│   ├── ast.rs               # Abstract Syntax Tree (AST) definitions
│   ├── token.rs             # Definisi token & source spans
│   ├── lexer.rs             # Tokenizer / Lexical analyzer
│   ├── parser.rs            # Recursive descent parser
│   ├── sema/                # Analisis semantik & tabel simbol
│   │   ├── mod.rs           # Type checking & scope verification
│   │   ├── symbols.rs       # Tabel simbol, scope, dan konstanta
│   │   └── types.rs         # Type compatibility & promotions
│   ├── codegen/             # Generator kode assembly x86 GNU
│   │   ├── mod.rs
│   │   ├── runtime.rs       # Runtime glue (_start & syscall wrappers)
│   │   └── x86_gas.rs       # Assembly emitter (Intel syntax noprefix)
│   ├── driver.rs            # Orkestrasi kompilasi, gas (as), dan ld
│   ├── error.rs             # Format diagnostik error dengan caret
│   ├── target.rs            # Konfigurasi target (nyxara-x86, linux-x86)
│   ├── lib.rs
│   └── main.rs              # CLI interface
├── runtime/
│   └── user.ld              # Linker script ELF32 untuk userland NyxaraOS
├── lib/
│   └── nyx/
│       └── sys.nyx          # Header pustaka standar syscall NyxaraOS
├── Learning-NyxC/           # Kumpulan contoh kode dan tutorial belajar
├── docs/                    # Dokumentasi arsitektur & spesifikasi bahasa
└── tests/
    └── compile_tests.rs     # Unit test dan integrasi end-to-end
```

---

## Instalasi & Persyaratan

Pastikan dependensi sistem berikut terpasang di host Linux:
- **Rust & Cargo** (1.70+)
- **GNU Assembler (`as`)** dengan dukungan 32-bit (`--32`)
- **GNU Linker (`ld`)** dengan emulasi `elf_i386`

Di distro berbasis Arch / Gentoo:
```bash
# Gentoo / Arch (biasanya paket binutils sudah mendukung x86 32-bit)
as --version
ld -V | grep elf_i386
```

---

## Panduan Penggunaan CLI

### 1. Memeriksa Sintaks & Semantik (Check)
Untuk memverifikasi kebenaran tipe dan sintaks tanpa kompilasi ke biner:
```bash
cargo run -- check <file.nyx>
```

### 2. Mengeluarkan Berkas Assembly x86 (`.s`)
```bash
cargo run -- emit-asm <file.nyx> -o <output.s>
```

### 3. Mengompilasi Menjadi Biner ELF32 NyxaraOS
```bash
cargo run -- build <file.nyx> -o <output.elf>
```
Opsi tambahan:
- `-k`, `--keep-temps`: Mempertahankan file sementara (`.s` dan `.o`).
- `--target <triple>`: Memilih target (default: `nyxara-x86`).

---

## Contoh Kode NyxC

### Hello World (`hello.nyx`)
```nyx
import "nyx/sys"

fn main() -> i32 {
    sys::write(1, "Halo dari NyxC!\n", str_len("Halo dari NyxC!\n"))
    return 0
}
```

### Menggunakan Builtin Print & Perulangan For
```nyx
fn main() -> i32 {
    let mut total = 0
    for (let mut i = 1; i <= 5; i = i + 1) {
        total = total + i
    }

    print("Hasil penjumlahan: ")
    println(itoa(total))
    return 0
}
```

---

## Menjalankan Pengujian

Menjalankan seluruh unit test dan integrasi compiler:
```bash
cargo test
```
Menjalankan pengujian mode rilis:
```bash
cargo test --release
```

---

## Materi Latihan
Panduan dan latihan bertahap tersedia di direktori [`Learning-NyxC/`](file:///home/akrom/Projects/NyxC/Learning-NyxC/README.md).
