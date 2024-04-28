# Featherweight muse

An implementation of a featherweight muse interpreter written in Rust.

Featherweight muse is a simple language that is a subset of muse. It is designed to test the type system and the evaluation of muse, including compile-time memory management through borrow checking and local type inference.

Currently, featherweight muse supports the following:
- [x] Variable declarations
- [x] Function declarations
- [x] Function calls
- [x] Assignment
- [x] Heap allocation
- [x] Mutable/Immutable references
- [x] Borrow checking system
- [x] Ownership system
- [x] Lifetime system
- [x] Auto-dereferencing
- [ ] Local type inference

## Getting Started

### Pre-requisites
- [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### Install
```bash
cargo install
``` 

### Usage

```bash
cargo run -- <options> <file>
```

### Options
```
Options:
        -h, -help       Display help
        -l, -lex        Display lexer output
        -p, -parse      Display parser output
        -t, -typecheck  Display typecheck output
        -e, -eval       Display eval output
```

### Test
```bash
cargo test
```

## Example

```bash
cargo run tests/swap.mu
```

## Syntax
```rust
fn swap(mut ref a: int, mut ref b: int) {
    let mut tmp = a
    a = b
    b = tmp
}

let mut x = 10
let mut y = 20

swap(x, y)

let mut heap_x = box x
```
