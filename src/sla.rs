use std::{collections::HashMap, io::Cursor, sync::Mutex};

macro_rules! sla_list{
    ($($name: literal),*) => {
        const SLA_LIST: &[(&str, &[u8])] = &[
            $(
                ($name, include_bytes!(concat!("../sla/", $name, ".sla.xz"))),
            )*
        ];
    }
}

sla_list!(
    "6502",
    "68020",
    "68030",
    "68040",
    "6805",
    "6809",
    "80251",
    "80390",
    "8048",
    "8051",
    "8085",
    "AARCH64BE",
    "AARCH64",
    "ARM4_be",
    "ARM4_le",
    "ARM4t_be",
    "ARM4t_le",
    "ARM5_be",
    "ARM5_le",
    "ARM5t_be",
    "ARM5t_le",
    "ARM6_be",
    "ARM6_le",
    "ARM7_be",
    "ARM7_le",
    "ARM8_be",
    "ARM8_le",
    "avr32a",
    "avr8eind",
    "avr8e",
    "avr8",
    "avr8xmega",
    "coldfire",
    "CP1600",
    "CR16B",
    "CR16C",
    "Dalvik",
    "data-be-64",
    "data-le-64",
    "dsPIC30F",
    "dsPIC33C",
    "dsPIC33E",
    "dsPIC33F",
    "HC05",
    "HC08",
    "HCS08",
    "HCS12",
    "JVM",
    "m8c",
    "MCS96",
    "mips32be",
    "mips32le",
    "mips32R6be",
    "mips32R6le",
    "mips64be",
    "mips64le",
    "mx51",
    "pa-risc32be",
    "pic12c5xx",
    "pic16c5x",
    "pic16f",
    "pic16",
    "pic17c7xx",
    "pic18",
    "PIC24E",
    "PIC24F",
    "PIC24H",
    "ppc_32_4xx_be",
    "ppc_32_4xx_le",
    "ppc_32_be",
    "ppc_32_le",
    "ppc_32_quicciii_be",
    "ppc_32_quicciii_le",
    "ppc_64_be",
    "ppc_64_isa_altivec_be",
    "ppc_64_isa_altivec_le",
    "ppc_64_isa_altivec_vle_be",
    "ppc_64_isa_be",
    "ppc_64_isa_le",
    "ppc_64_isa_vle_be",
    "ppc_64_le",
    "riscv.ilp32d",
    "riscv.lp64d",
    "sh-1",
    "sh-2a",
    "sh-2",
    "SparcV9_32",
    "SparcV9_64",
    "SuperH4_be",
    "SuperH4_le",
    "TI_MSP430",
    "TI_MSP430X",
    "toy64_be_harvard",
    "toy64_be",
    "toy64_le",
    "toy_be_posStack",
    "toy_be",
    "toy_builder_be_align2",
    "toy_builder_be",
    "toy_builder_le_align2",
    "toy_builder_le",
    "toy_le",
    "toy_wsz_be",
    "toy_wsz_le",
    "tricore",
    "V850",
    "x86-64",
    "x86",
    "z180",
    "z80"
);

lazy_static::lazy_static! {
    static ref SPEC_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn get_arch_sla(arch: &str) -> Option<String> {
    if let Some(spec) = SPEC_CACHE.lock().unwrap().get(arch) {
        return Some(spec.to_string());
    }

    let data = SLA_LIST.iter().find(|(a, _)| *a == arch).map(|(_, b)| b)?;

    let mut input = Cursor::new(*data);
    let mut buf = vec![];
    let mut output = Cursor::new(&mut buf);
    lzma_rs::xz_decompress(&mut input, &mut output).unwrap();

    let spec = String::from_utf8(buf).unwrap();
    SPEC_CACHE
        .lock()
        .unwrap()
        .insert(arch.to_string(), spec.clone());
    Some(spec)
}
