# Sleigh

Sleigh decompiler

## Quickstart

Add the following to Cargo.toml:

```toml
sleigh = "*"
```

Create a decompiler and decompile any bytecodes you want:

```rust
let mut decompiler = Decompiler::builder().x86(X86Mode::Mode32).build();

let code = b"\x01\xd8"; // add eax, ebx

let (len, pcodes) = decompiler.translate(&code, 0x1000);
println!("{} {:?}", len, pcodes);

let (len, insts) = decompiler.decompile(&code, 0x1000);
println!("{} {:?}", len, insts);
```

