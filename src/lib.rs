use std::{collections::HashMap, pin::Pin};

use cxx::{let_cxx_string, UniquePtr};
use sleigh_sys::{RustAssemblyEmit, RustLoadImage, RustPCodeEmit};

pub mod sla;

pub type Opcode = sleigh_sys::Opcode;
pub type SpaceType = sleigh_sys::SpaceType;

#[derive(Debug)]
pub struct AddrSpace {
    pub name: String,
    pub ty: SpaceType,
}

#[derive(Debug)]
pub struct VarnodeData {
    pub space: AddrSpace,
    pub offset: u64,
    pub size: u32,
}

impl From<&sleigh_sys::ffi::VarnodeData> for VarnodeData {
    fn from(var: &sleigh_sys::ffi::VarnodeData) -> Self {
        let address = sleigh_sys::ffi::getVarnodeDataAddress(var);
        let offset = address.getOffset();
        let space = address.getSpace();
        let space = unsafe {
            let space = &*space;
            let ty = sleigh_sys::ffi::getAddrSpaceType(space);
            let ty = sleigh_sys::SpaceType::from_u32(ty).unwrap();
            let name = space.getName().to_string();
            AddrSpace { name, ty }
        };
        let size = sleigh_sys::ffi::getVarnodeSize(var);
        Self {
            space,
            offset,
            size,
        }
    }
}

#[derive(Debug)]
pub struct PCode {
    pub address: u64,
    pub opcode: Opcode,
    pub vars: Vec<VarnodeData>,
    pub outvar: Option<VarnodeData>,
}

#[derive(Debug)]
pub struct Instruction {
    pub address: u64,
    pub mnemonic: String,
    pub body: String,
}

struct AssemblyEmit {
    insts: Vec<Instruction>,
}

impl sleigh_sys::AssemblyEmit for AssemblyEmit {
    fn dump(&mut self, addr: &sleigh_sys::ffi::Address, mnem: &str, body: &str) {
        let address = addr.getOffset();
        let mnemonic = mnem.to_string();
        let body = body.to_string();
        self.insts.push(Instruction {
            address,
            mnemonic,
            body,
        });
    }
}

struct PCodeEmit {
    pcodes: Vec<PCode>,
}

impl sleigh_sys::PCodeEmit for PCodeEmit {
    fn dump(
        &mut self,
        address: &sleigh_sys::ffi::Address,
        opcode: sleigh_sys::Opcode,
        outvar: Option<&sleigh_sys::ffi::VarnodeData>,
        vars: &[sleigh_sys::ffi::VarnodeData],
    ) {
        let vars = vars.iter().map(VarnodeData::from).collect::<Vec<_>>();
        let outvar = outvar.map(VarnodeData::from);
        let address = address.getOffset();
        let pcode = PCode {
            address,
            opcode,
            vars,
            outvar,
        };
        self.pcodes.push(pcode);
    }
}

struct SliceLoader<'a> {
    start: u64,
    data: &'a [u8],
}

impl<'a> sleigh_sys::LoadImage for SliceLoader<'a> {
    fn load_fill(&mut self, ptr: &mut [u8], addr: &sleigh_sys::ffi::Address) {
        let addr = addr.getOffset();
        let len = self.data.len() as u64;
        let required = ptr.len() as u64;
        ptr.fill(0);

        if self.start <= addr {
            let fill_len = required.min(len) as usize;
            let offset = (addr - self.start) as usize;
            ptr[..fill_len].copy_from_slice(&self.data[offset..offset + fill_len]);
        }
    }
}

struct VectorLoader {
    start: u64,
    data: Vec<u8>,
}

impl sleigh_sys::LoadImage for VectorLoader {
    fn load_fill(&mut self, ptr: &mut [u8], addr: &sleigh_sys::ffi::Address) {
        let mut s = SliceLoader {
            start: self.start,
            data: &self.data,
        };
        s.load_fill(ptr, addr);
    }
}

pub enum X86Mode {
    Mode16,
    Mode32,
    Mode64,
}

pub enum X64Mode {
    Mode16,
    Mode32,
    Mode64,
}

pub enum ArmMode {
    Arm,
    Thumb,
}

pub enum ArmVersion {
    Arm4,
    Arm4t,
    Arm5,
    Arm5t,
    Arm6,
    Arm7,
    Arm8,
}

pub enum Endian {
    LittleEndian,
    BigEndian,
}

pub struct Image {
    pub base_addr: u64,
    pub data: Vec<u8>,
}

pub struct ArchState {
    spec: String,
    var: HashMap<String, u32>,
}

pub struct DecompilerBuilder<T> {
    state: T,
}

impl DecompilerBuilder<()> {
    pub fn x86(self, mode: X86Mode) -> DecompilerBuilder<ArchState> {
        let mut var = HashMap::new();
        let m = match mode {
            X86Mode::Mode16 => 0,
            X86Mode::Mode32 => 1,
            X86Mode::Mode64 => 2,
        };
        var.insert("addrsize".to_string(), m);
        var.insert("opsize".to_string(), m);
        let spec = match mode {
            X86Mode::Mode16 | X86Mode::Mode32 => sla::get_arch_sla("x86").unwrap(),
            X86Mode::Mode64 => sla::get_arch_sla("x86-64").unwrap(),
        };
        DecompilerBuilder {
            state: ArchState { spec, var },
        }
    }

    pub fn aarch64(self, endian: Endian) -> DecompilerBuilder<ArchState> {
        let e = match endian {
            Endian::LittleEndian => "",
            Endian::BigEndian => "BE",
        };

        let name = format!("AARCH64{}", e);
        let spec = sla::get_arch_sla(&name).unwrap();

        DecompilerBuilder {
            state: ArchState {
                spec,
                var: HashMap::new(),
            },
        }
    }

    pub fn arm(
        self,
        version: ArmVersion,
        endian: Endian,
        mode: ArmMode,
    ) -> DecompilerBuilder<ArchState> {
        let v = match version {
            ArmVersion::Arm4 => "4",
            ArmVersion::Arm5 => "5",
            ArmVersion::Arm6 => "6",
            ArmVersion::Arm7 => "7",
            ArmVersion::Arm8 => "8",
            ArmVersion::Arm4t => "4t",
            ArmVersion::Arm5t => "5t",
        };
        let e = match endian {
            Endian::LittleEndian => "le",
            Endian::BigEndian => "be",
        };

        let mut var = HashMap::new();
        let t = if let ArmMode::Thumb = mode { 1 } else { 0 };
        var.insert("TMode".to_string(), t);

        let name = format!("ARM{}_{}", v, e);
        let spec = sla::get_arch_sla(&name).unwrap();

        DecompilerBuilder {
            state: ArchState { spec, var },
        }
    }

    pub fn dalvik(self) -> DecompilerBuilder<ArchState> {
        DecompilerBuilder {
            state: ArchState {
                spec: sla::get_arch_sla("Dalvik").unwrap(),
                var: HashMap::new(),
            },
        }
    }

    pub fn jvm(self) -> DecompilerBuilder<ArchState> {
        DecompilerBuilder {
            state: ArchState {
                spec: sla::get_arch_sla("JVM").unwrap(),
                var: HashMap::new(),
            },
        }
    }
}

impl DecompilerBuilder<ArchState> {
    pub fn build(self) -> Decompiler {
        let_cxx_string!(spec = self.state.spec);
        let doc = sleigh_sys::ffi::newDocumentStorage(&spec);
        let loader = VectorLoader {
            start: 0,
            data: vec![],
        };
        let mut loader = Box::new(loader);

        unsafe {
            let rust_loader = Box::new(RustLoadImage::from_internal(std::mem::transmute::<
                _,
                &'static mut VectorLoader,
            >(loader.as_mut())));
            let rust_loader: *mut RustLoadImage<'static> = Box::leak(rust_loader);
            let mut inner = sleigh_sys::ffi::newDecompiler(rust_loader, doc);

            let ctx = inner.pin_mut().getContext();
            for (k, v) in self.state.var.iter() {
                let_cxx_string!(key = k);
                let val = *v;
                Pin::new_unchecked(&mut *ctx).setVariableDefault(&key, val)
            }

            Decompiler {
                loader,
                rust_loader,
                inner,
            }
        }
    }
}

pub struct Decompiler {
    loader: Box<VectorLoader>,
    rust_loader: *mut RustLoadImage<'static>,
    inner: UniquePtr<sleigh_sys::ffi::Decompiler>,
}

impl Drop for Decompiler {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.rust_loader);
        }
    }
}

impl Decompiler {
    pub fn builder() -> DecompilerBuilder<()> {
        DecompilerBuilder { state: () }
    }

    pub fn translate(&mut self, code: &[u8], addr: u64) -> (usize, Vec<PCode>) {
        self.loader.data.clear();
        self.loader.data.extend_from_slice(code);
        self.loader.start = addr;
        let mut emit = PCodeEmit { pcodes: vec![] };
        unsafe {
            let mut rust_emit = RustPCodeEmit::from_internal(&mut emit);
            let n = self
                .inner
                .pin_mut()
                .translate(&mut rust_emit as *mut _, addr);
            (n as usize, emit.pcodes)
        }
    }

    pub fn disassemble(&mut self, code: &[u8], addr: u64) -> (usize, Vec<Instruction>) {
        self.loader.data.clear();
        self.loader.data.extend_from_slice(code);
        self.loader.start = addr;
        let mut emit = AssemblyEmit { insts: vec![] };
        unsafe {
            let mut rust_emit = RustAssemblyEmit::from_internal(&mut emit);
            let n = self.inner.pin_mut().disassemble(&mut rust_emit as _, addr);
            (n as usize, emit.insts)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop() {
        let mut decompiler = Decompiler::builder()
            .arm(ArmVersion::Arm8, Endian::LittleEndian, ArmMode::Arm)
            .build();
        for _ in 0..100 {
            let (n, pcodes) = decompiler.translate(b"\x01\x00\x80\x00", 0x1000);
            println!("{} {:?}", n, pcodes);
            let (n, insts) = decompiler.disassemble(b"\x01\x00\x80\x00", 0x1000);
            println!("{} {:?}", n, insts);
        }
    }

    fn run(decompiler: &mut Decompiler, code: &[u8], addr: u64) {
        let (n, pcodes) = decompiler.translate(code, addr);
        println!("{} {:?}", n, pcodes);
        let (n, insts) = decompiler.disassemble(code, addr);
        println!("{} {:?}", n, insts);
    }

    #[test]
    fn test_concurrent() {
        let a = std::thread::spawn(test_x86);
        let b = std::thread::spawn(test_arm);
        a.join().unwrap();
        b.join().unwrap();
    }

    #[test]
    fn test_x86() {
        let mut decompiler = Decompiler::builder().x86(X86Mode::Mode32).build();
        run(&mut decompiler, b"\x05\x00\x10\x00\x00", 0x1000);
        let mut decompiler = Decompiler::builder().x86(X86Mode::Mode64).build();
        run(&mut decompiler, b"\x48\x31\xd8", 0x100010001);
    }

    #[test]
    fn test_arm() {
        let mut decompiler = Decompiler::builder()
            .arm(ArmVersion::Arm8, Endian::LittleEndian, ArmMode::Arm)
            .build();
        run(&mut decompiler, b"\x01\x00\x80\x00", 0x1000);
    }

    #[test]
    fn test_arm_thumb() {
        let mut decompiler = Decompiler::builder()
            .arm(ArmVersion::Arm5t, Endian::LittleEndian, ArmMode::Thumb)
            .build();
        run(&mut decompiler, b"\x11\x44\x11\x44", 0x1000);
    }

    #[test]
    fn test_dalvik() {
        let mut decompiler = Decompiler::builder().dalvik().build();
        run(&mut decompiler, b"\x90\x00\x02\x03", 0x1000);
    }
}
