# Panduan Belajar Bahasa Pemrograman Nyx (.nyx)

Folder ini berisi materi latihan dasar untuk memahami sintaksis dan fitur bahasa **NyxC** langkah demi langkah.

---

## 📚 Daftar Materi & Contoh

| File | Topik | Konsep yang Dipelajari |
|---|---|---|
| [`01_hello.nyx`](file:///home/akrom/Projects/NyxC/belajar/01_hello.nyx) | Hello World | Struktur fungsi `main`, `import "nyx/sys"`, dan output teks |
| [`02_variabel.nyx`](file:///home/akrom/Projects/NyxC/belajar/02_variabel.nyx) | Variabel & Konstanta | `let` (immutable), `let mut` (mutable), dan `const` |
| [`03_aritmatika.nyx`](file:///home/akrom/Projects/NyxC/belajar/03_aritmatika.nyx) | Aritmatika | Operator `+`, `-`, `*`, `/`, `%` pada tipe `i32` |
| [`04_if_else.nyx`](file:///home/akrom/Projects/NyxC/belajar/04_if_else.nyx) | Percabangan | Logika `if`, `else if`, `else`, dan operator perbandingan (`>=`, `<`, `==`) |
| [`05_while.nyx`](file:///home/akrom/Projects/NyxC/belajar/05_while.nyx) | Perulangan | Loop `while` dengan variabel counter `mut` |
| [`06_fungsi.nyx`](file:///home/akrom/Projects/NyxC/belajar/06_fungsi.nyx) | Fungsi Kustom | Definisi `fn name(params) -> return_type` dan pemanggilan fungsi |
| [`07_syscall.nyx`](file:///home/akrom/Projects/NyxC/belajar/07_syscall.nyx) | Syscall Nyxara OS | Interaksi syscall kernel (seperti `sys::write`) |

---

## 🛠️ Cara Mengompilasi Kode `.nyx`

Dari direktori root `NyxC/`:

1. **Pastikan folder `build/` sudah dibuat:**
   ```bash
   mkdir -p build
   ```

2. **Kompilasi langsung menjadi Executable binary (ELF 32-bit):**
   ```bash
   cargo run -- build Learning-NyxC/01_hello.nyx -o build/01_hello
   ```

3. **(Opsional) Jika hanya ingin menghasilkan Assembly x86 (`.s`):**
   ```bash
   cargo run -- emit-asm Learning-NyxC/01_hello.nyx -o build/01_hello.s
   ```

4. **(Opsional) Verifikasi sintaks tanpa compile/link:**
   ```bash
   cargo run -- check Learning-NyxC/01_hello.nyx
   ```

---

## 💡 Ringkasan Tipe Data NyxC

- **Integer Bertanda (Signed):** `i8`, `i16`, `i32`
- **Integer Tanpa Tanda (Unsigned):** `u8`, `u16`, `u32`
- **Boolean:** `bool` (`true` / `false`)
- **Pointer:** `*T` (contoh: `*i32`, `*u8`)
- **Void:** `void` (tidak mengembalikan nilai)
