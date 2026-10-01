# NyxC Language Specification (v0.1)

## 1. Syntax & Lexical Conventions

### Comments
- Line comment: `// Single-line comment`
- Block comment: `/* Multi-line comment */`

### Semicolons
Semicolons (`;`) are optional. Statements may be terminated by explicit semicolons or newline boundaries.

### Identifiers
`[a-zA-Z_][a-zA-Z0-9_]*`

## 2. Primitive Types
- Integers: `i8`, `i16`, `i32`, `u8`, `u16`, `u32`
- Boolean: `bool` (`true`, `false`)
- Void: `void`
- Pointers: `*T` (e.g. `*u8`, `*i32`, `**u8`)

## 3. Declarations

### Functions
```nyx
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

fn log_msg(msg: *u8) {
    sys::write(1, msg, 12)
}
```

### Variables & Constants
```nyx
let a: i32 = 10
let mut counter: i32 = 0
const BUFFER_SIZE: i32 = 1024
```

### External Declarations
```nyx
extern fn sys_write(fd: i32, buf: *u8, len: i32) -> i32
```

## 4. Control Flow

### If / Else
```nyx
if x > 0 {
    sys::write(1, "positive\n", 9)
} else {
    sys::write(1, "zero or negative\n", 17)
}
```

### While
```nyx
while counter < 10 {
    counter = counter + 1
}
```

## 5. Operators
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- Bitwise: `&`, `|`, `^`, `~`, `<<`, `>>`
- Pointer operations: `&x` (address-of), `*ptr` (dereference)

## 6. Built-in Syscall Intrinsic
NyxC provides direct low-level hardware interrupt dispatch via `syscall`:
```nyx
syscall(4, 1, msg, len) // eax=4 (SYS_WRITE), ebx=1, ecx=msg, edx=len
```
Or via standard library module `nyx/sys`:
```nyx
import "nyx/sys"

fn main() -> i32 {
    sys::write(1, "Hello from NyxC!\n", 17)
    return 0
}
```
