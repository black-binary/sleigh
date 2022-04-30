use sleigh::{Decompiler, X86Mode};

fn main() {
    let mut decompiler = Decompiler::builder().x86(X86Mode::Mode32).build();

    let code = b"\x01\xd8"; // ADD EAX, EBX

    // Lift bytecodes into SLEIGH IL
    let (len, pcodes) = decompiler.translate(code, 0x1000);
    println!("{} {:?}", len, pcodes);

    // Disasm bytecodes
    let (len, insts) = decompiler.disassemble(code, 0x1000);
    println!("{} {:?}", len, insts);
}
