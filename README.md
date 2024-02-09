# Featherweight muse

An implementation of a featherweight muse compiler written in Rust.

## Pre-requisites
- [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Install
```bash
cargo install
```

## Usage

```bash
cargo run <options> <file>
```

### Options
```
Options:
        -h, -help       Display this message
        -l, -lex        Enable lexer output
        -p, -parse      Enable parser output
        -t, -typecheck  Enable typecheck output
        -e, -eval       Enable eval output
```

## Example

```bash
cargo run tests/test.mu
```
