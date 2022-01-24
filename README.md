# SLEIGH

Rust port of Ghidra's SLEIGH decompiler. This library allows you to decompile or translate machine code for multiple architectures.

## What is SLEIGH?

SLEIGH is a language for describing the instruction sets of general purpose microprocessors, in order to facilitate the reverse engineering of software written for them. SLEIGH was designed for the GHIDRA reverse engineering platform and is used to describe microprocessors with enough detail to facilitate two major components of GHIDRA, the disassembly and decompilation engines. 

## Quickstart

Add the following to Cargo.toml:

```toml
sleigh = "*"
```

Create a decompiler and decompile bytecodes:

```rust
let mut decompiler = Decompiler::builder().x86(X86Mode::Mode32).build();

let code = b"\x01\xd8"; // ADD EAX, EBX

// Lift bytecodes into P-Code
let (len, pcodes) = decompiler.translate(&code, 0x1000);
println!("{} {:?}", len, pcodes);

// Disasm bytecodes
let (len, insts) = decompiler.disassemble(&code, 0x1000);
println!("{} {:?}", len, insts);
```

## Supported Architectures

| Arch | State |
| - | -  |
| x86 | ✔️ |
| x86_64 | ✔️|
| ARM(v4/5/6/7/8/thumb) | ✔️ |
| AArch64 | ✔️ |
| MIPS | 🚧 |
| PowerPC | 🚧 |
| AVR | 🚧 |
| Dalvik | ✔️ |
| JVM | ✔️ |
